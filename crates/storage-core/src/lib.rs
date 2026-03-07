use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app not found")]
    AppNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app_id: String,
    pub label: String,
    pub token_ttl_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeRecord {
    pub nonce: String,
    pub public_key_hint: String,
    pub expires_at_ms: i64,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRecord {
    pub token_id: String,
    pub token_hash_hex: String,
    pub public_key_hex: String,
    pub app_id: String,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub revoked: bool,
}

#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub password: String,
    pub record: TokenRecord,
}

#[derive(Debug, Clone)]
pub struct FileStore {
    data_dir: PathBuf,
}

impl FileStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let store = Self {
            data_dir: data_dir.into(),
        };
        store.ensure_layout()?;
        Ok(store)
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn ensure_layout(&self) -> Result<(), StorageError> {
        fs::create_dir_all(self.challenges_dir())?;
        fs::create_dir_all(self.tokens_dir())?;
        fs::create_dir_all(self.accounts_dir())?;
        Ok(())
    }

    pub fn load_app_configs(config_path: &Path) -> Result<Vec<AppConfig>, StorageError> {
        let raw = fs::read_to_string(config_path)?;
        serde_json::from_str(&raw).map_err(StorageError::from)
    }

    pub fn save_challenge(&self, record: &ChallengeRecord) -> Result<(), StorageError> {
        fs::write(
            self.challenge_path(&record.nonce),
            serde_json::to_vec_pretty(record)?,
        )?;
        Ok(())
    }

    pub fn take_challenge(&self, nonce: &str) -> Result<Option<ChallengeRecord>, StorageError> {
        let path = self.challenge_path(nonce);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read(&path)?;
        fs::remove_file(&path)?;
        Ok(Some(serde_json::from_slice(&raw)?))
    }

    pub fn issue_token(
        &self,
        public_key_hex: &str,
        app_id: &str,
        created_at_ms: i64,
        expires_at_ms: i64,
    ) -> Result<IssuedToken, StorageError> {
        let token_id = Uuid::new_v4().to_string();
        let password = random_secret()?;
        let record = TokenRecord {
            token_id: token_id.clone(),
            token_hash_hex: sha256_hex(&password),
            public_key_hex: public_key_hex.to_string(),
            app_id: app_id.to_string(),
            created_at_ms,
            expires_at_ms,
            revoked: false,
        };
        fs::create_dir_all(self.account_app_dir(public_key_hex, app_id))?;
        fs::write(
            self.token_path(&token_id),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(IssuedToken { password, record })
    }

    pub fn authenticate_token(
        &self,
        public_key_hex: &str,
        app_id: &str,
        password: &str,
        now_ms: i64,
    ) -> Result<Option<TokenRecord>, StorageError> {
        let expected_hash = sha256_hex(password);
        for entry in fs::read_dir(self.tokens_dir())? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                continue;
            }
            let raw = fs::read(entry.path())?;
            let token = serde_json::from_slice::<TokenRecord>(&raw)?;
            if token.public_key_hex == public_key_hex
                && token.app_id == app_id
                && token.token_hash_hex == expected_hash
                && !token.revoked
                && token.expires_at_ms >= now_ms
            {
                return Ok(Some(token));
            }
        }
        Ok(None)
    }

    pub fn account_app_dir(&self, public_key_hex: &str, app_id: &str) -> PathBuf {
        self.accounts_dir()
            .join(public_key_hex)
            .join("apps")
            .join(app_id)
    }

    fn challenges_dir(&self) -> PathBuf {
        self.data_dir.join("challenges")
    }

    fn tokens_dir(&self) -> PathBuf {
        self.data_dir.join("tokens")
    }

    fn accounts_dir(&self) -> PathBuf {
        self.data_dir.join("accounts")
    }

    fn challenge_path(&self, nonce: &str) -> PathBuf {
        self.challenges_dir().join(format!("{nonce}.json"))
    }

    fn token_path(&self, token_id: &str) -> PathBuf {
        self.tokens_dir().join(format!("{token_id}.json"))
    }
}

fn random_secret() -> Result<String, StorageError> {
    let mut bytes = [0u8; 24];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("getrandom failed: {error}"),
        )
    })?;
    Ok(hex::encode(bytes))
}

fn sha256_hex(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::FileStore;

    fn temp_dir() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("dweb-cloud-test-{stamp}"))
    }

    #[test]
    fn issues_and_authenticates_token() {
        let store = FileStore::new(temp_dir()).unwrap();
        let issued = store.issue_token("pub", "gaubee-2fa", 1, 10_000).unwrap();
        let auth = store
            .authenticate_token("pub", "gaubee-2fa", &issued.password, 2)
            .unwrap();
        assert!(auth.is_some());
    }
}
