use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("secret input is empty")]
    EmptyInput,
    #[error("mnemonic generation failed")]
    MnemonicGeneration,
    #[error("public key is invalid")]
    InvalidPublicKey,
    #[error("signature is invalid")]
    InvalidSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMaterial {
    pub mnemonic: String,
    pub public_key_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureChallenge {
    pub app_id: String,
    pub nonce: String,
    pub timestamp_ms: i64,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedChallenge {
    pub public_key_hex: String,
    pub signature_hex: String,
    pub app_id: String,
    pub timestamp_ms: i64,
    pub device_id: String,
    pub nonce: String,
}

pub fn derive_identity(secret_input: &str) -> Result<IdentityMaterial, IdentityError> {
    let mnemonic = parse_or_derive_mnemonic(secret_input)?;
    let signing_key = signing_key_from_mnemonic(&mnemonic);
    let verifying_key = signing_key.verifying_key();
    Ok(IdentityMaterial {
        mnemonic: mnemonic.to_string(),
        public_key_hex: hex::encode(verifying_key.to_bytes()),
    })
}

pub fn sign_challenge(
    secret_input: &str,
    challenge: &SignatureChallenge,
) -> Result<SignedChallenge, IdentityError> {
    let mnemonic = parse_or_derive_mnemonic(secret_input)?;
    let signing_key = signing_key_from_mnemonic(&mnemonic);
    let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let message = signature_message(&public_key_hex, challenge);
    let signature = signing_key.sign(message.as_bytes());
    Ok(SignedChallenge {
        public_key_hex,
        signature_hex: hex::encode(signature.to_bytes()),
        app_id: challenge.app_id.clone(),
        timestamp_ms: challenge.timestamp_ms,
        device_id: challenge.device_id.clone(),
        nonce: challenge.nonce.clone(),
    })
}

pub fn verify_signed_challenge(challenge: &SignedChallenge) -> Result<(), IdentityError> {
    let public_key_bytes: [u8; 32] = hex::decode(&challenge.public_key_hex)
        .map_err(|_| IdentityError::InvalidPublicKey)?
        .try_into()
        .map_err(|_| IdentityError::InvalidPublicKey)?;
    let signature_bytes: [u8; 64] = hex::decode(&challenge.signature_hex)
        .map_err(|_| IdentityError::InvalidSignature)?
        .try_into()
        .map_err(|_| IdentityError::InvalidSignature)?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| IdentityError::InvalidPublicKey)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let message = signature_message(
        &challenge.public_key_hex,
        &SignatureChallenge {
            app_id: challenge.app_id.clone(),
            nonce: challenge.nonce.clone(),
            timestamp_ms: challenge.timestamp_ms,
            device_id: challenge.device_id.clone(),
        },
    );
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| IdentityError::InvalidSignature)
}

pub fn signature_message(public_key_hex: &str, challenge: &SignatureChallenge) -> String {
    format!(
        "dweb-cloud:{}:{}:{}:{}:{}",
        challenge.app_id,
        challenge.nonce,
        challenge.timestamp_ms,
        challenge.device_id,
        public_key_hex
    )
}

fn parse_or_derive_mnemonic(secret_input: &str) -> Result<Mnemonic, IdentityError> {
    let normalized = secret_input.trim();
    if normalized.is_empty() {
        return Err(IdentityError::EmptyInput);
    }
    if let Ok(mnemonic) = Mnemonic::parse_in(Language::English, normalized) {
        return Ok(mnemonic);
    }
    let digest = Sha256::digest(normalized.as_bytes());
    Mnemonic::from_entropy(&digest).map_err(|_| IdentityError::MnemonicGeneration)
}

fn signing_key_from_mnemonic(mnemonic: &Mnemonic) -> SigningKey {
    let seed = Zeroizing::new(mnemonic.to_seed_normalized(""));
    let mut signing_key_bytes = [0u8; 32];
    signing_key_bytes.copy_from_slice(&seed[..32]);
    SigningKey::from_bytes(&signing_key_bytes)
}

#[cfg(test)]
mod tests {
    use super::{SignatureChallenge, derive_identity, sign_challenge, verify_signed_challenge};

    #[test]
    fn derives_stable_identity() {
        let left = derive_identity("secret").unwrap();
        let right = derive_identity("secret").unwrap();
        assert_eq!(left.public_key_hex, right.public_key_hex);
        assert_eq!(left.mnemonic, right.mnemonic);
    }

    #[test]
    fn signs_and_verifies() {
        let signed = sign_challenge(
            "secret",
            &SignatureChallenge {
                app_id: "gaubee-2fa".into(),
                nonce: "nonce-1".into(),
                timestamp_ms: 1,
                device_id: "device-1".into(),
            },
        )
        .unwrap();
        verify_signed_challenge(&signed).unwrap();
    }
}
