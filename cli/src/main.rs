use clap::{Parser, Subcommand};
use dweb_cloud_identity_core::{SignatureChallenge, derive_identity, sign_challenge};
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Token(TokenCommand),
}

#[derive(Debug, Parser)]
struct TokenCommand {
    #[command(subcommand)]
    command: TokenSubcommand,
}

#[derive(Debug, Subcommand)]
enum TokenSubcommand {
    Issue(TokenIssueArgs),
}

#[derive(Debug, Parser)]
struct TokenIssueArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    app: String,
    #[arg(long)]
    secret: String,
    #[arg(long, default_value = "cli-device")]
    device_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeResponse {
    nonce: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenIssueRequest {
    public_key_hex: String,
    signature_hex: String,
    app_id: String,
    timestamp_ms: i64,
    device_id: String,
    nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenIssueResponse {
    app_id: String,
    webdav_base_url: String,
    username: String,
    password: String,
    expires_at_ms: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Token(token) => match token.command {
            TokenSubcommand::Issue(args) => issue_token(args).await?,
        },
    }
    Ok(())
}

async fn issue_token(args: TokenIssueArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let identity = derive_identity(&args.secret)?;
    let challenge_url = args.server.join("/api/v1/auth/challenge")?;
    let challenge = client
        .post(challenge_url)
        .json(&serde_json::json!({ "publicKeyHint": identity.public_key_hex }))
        .send()
        .await?
        .error_for_status()?
        .json::<ChallengeResponse>()
        .await?;

    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let signed = sign_challenge(
        &args.secret,
        &SignatureChallenge {
            app_id: args.app.clone(),
            nonce: challenge.nonce.clone(),
            timestamp_ms,
            device_id: args.device_id.clone(),
        },
    )?;
    let issue_url = args
        .server
        .join(&format!("/api/v1/apps/{}/tokens", args.app))?;
    let response = client
        .post(issue_url)
        .json(&TokenIssueRequest {
            public_key_hex: signed.public_key_hex,
            signature_hex: signed.signature_hex,
            app_id: signed.app_id,
            timestamp_ms: signed.timestamp_ms,
            device_id: signed.device_id,
            nonce: signed.nonce,
        })
        .send()
        .await?
        .error_for_status()?
        .json::<TokenIssueResponse>()
        .await?;

    let webdav_base_url = args.server.join(&response.webdav_base_url)?.to_string();
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "appId": response.app_id,
                "webdavBaseUrl": webdav_base_url,
                "username": response.username,
                "password": response.password,
                "expiresAtMs": response.expires_at_ms,
            }))?
        );
    } else {
        println!("WebDAV host: {}", webdav_base_url);
        println!("WebDAV account: {}", response.username);
        println!("WebDAV password: {}", response.password);
        println!("Expires At: {}", response.expires_at_ms);
    }
    Ok(())
}
