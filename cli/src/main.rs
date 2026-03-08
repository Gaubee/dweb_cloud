use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use dweb_cloud_identity_core::{
    SignatureChallenge, SignedChallenge, derive_identity, sign_challenge,
};
use dweb_cloud_storage_core::{
    AccountOverview, AppConfig, FileStore, PlanConfig, StoreStats, TokenRecord,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

const ACCOUNT_MANAGEMENT_SCOPE: &str = "dweb-cloud-account";

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Token(TokenCommand),
    Account(AccountCommand),
    Public(PublicCommand),
    Developer(DeveloperCommand),
    Admin(AdminCommand),
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
struct AccountCommand {
    #[command(subcommand)]
    command: AccountSubcommand,
}

#[derive(Debug, Subcommand)]
enum AccountSubcommand {
    Overview(AccountOverviewArgs),
    Tokens(AccountTokensCommand),
}

#[derive(Debug, Parser)]
struct AccountTokensCommand {
    #[command(subcommand)]
    command: AccountTokensSubcommand,
}

#[derive(Debug, Subcommand)]
enum AccountTokensSubcommand {
    List(AccountTokenListArgs),
    Revoke(AccountTokenRevokeArgs),
}

#[derive(Debug, Parser)]
struct PublicCommand {
    #[command(subcommand)]
    command: PublicSubcommand,
}

#[derive(Debug, Subcommand)]
enum PublicSubcommand {
    Apps(PublicAppsArgs),
    Plans(PublicPlansArgs),
}

#[derive(Debug, Parser)]
struct DeveloperCommand {
    #[command(subcommand)]
    command: DeveloperSubcommand,
}

#[derive(Debug, Subcommand)]
enum DeveloperSubcommand {
    Meta(DeveloperMetaArgs),
}

#[derive(Debug, Parser)]
struct AdminCommand {
    #[command(subcommand)]
    command: AdminSubcommand,
}

#[derive(Debug, Subcommand)]
enum AdminSubcommand {
    Stats(AdminStatsArgs),
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

#[derive(Debug, Parser)]
struct AccountOverviewArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    secret: String,
    #[arg(long, default_value = "cli-device")]
    device_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct AccountTokenListArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    secret: String,
    #[arg(long)]
    app: Option<String>,
    #[arg(long, default_value = "cli-device")]
    device_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct AccountTokenRevokeArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    secret: String,
    #[arg(long)]
    token_id: String,
    #[arg(long, default_value = "cli-device")]
    device_id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct PublicAppsArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct PublicPlansArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct DeveloperMetaArgs {
    #[arg(long)]
    server: Url,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Parser)]
struct AdminStatsArgs {
    #[arg(long, default_value = "./.data")]
    data_dir: PathBuf,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountAuthRequest {
    public_key_hex: String,
    signature_hex: String,
    timestamp_ms: i64,
    device_id: String,
    nonce: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenListRequest {
    public_key_hex: String,
    signature_hex: String,
    timestamp_ms: i64,
    device_id: String,
    nonce: String,
    app_id_filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenListResponse {
    public_key_hex: String,
    tokens: Vec<TokenRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenRevokeResponse {
    token_id: String,
    revoked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeveloperMetaResponse {
    developer_mode: bool,
    management_scope_app_id: String,
    apps: Vec<AppConfig>,
    plans: Vec<PlanConfig>,
    stats: StoreStats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Token(token) => match token.command {
            TokenSubcommand::Issue(args) => issue_token(args).await?,
        },
        Command::Account(account) => match account.command {
            AccountSubcommand::Overview(args) => account_overview(args).await?,
            AccountSubcommand::Tokens(tokens) => match tokens.command {
                AccountTokensSubcommand::List(args) => account_token_list(args).await?,
                AccountTokensSubcommand::Revoke(args) => account_token_revoke(args).await?,
            },
        },
        Command::Public(public) => match public.command {
            PublicSubcommand::Apps(args) => public_apps(args).await?,
            PublicSubcommand::Plans(args) => public_plans(args).await?,
        },
        Command::Developer(developer) => match developer.command {
            DeveloperSubcommand::Meta(args) => developer_meta(args).await?,
        },
        Command::Admin(admin) => match admin.command {
            AdminSubcommand::Stats(args) => admin_stats(args)?,
        },
    }
    Ok(())
}

async fn issue_token(args: TokenIssueArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let identity = derive_identity(&args.secret)?;
    let challenge = fetch_challenge(&client, &args.server, &identity.public_key_hex).await?;
    let signed = sign_for_scope(&args.secret, &args.app, &challenge.nonce, &args.device_id)?;
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
        print_json(&serde_json::json!({
            "appId": response.app_id,
            "webdavBaseUrl": webdav_base_url,
            "username": response.username,
            "password": response.password,
            "expiresAtMs": response.expires_at_ms,
        }))?;
    } else {
        println!("WebDAV host: {}", webdav_base_url);
        println!("WebDAV account: {}", response.username);
        println!("WebDAV password: {}", response.password);
        println!("Expires At: {}", response.expires_at_ms);
    }
    Ok(())
}

async fn account_overview(args: AccountOverviewArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let request =
        build_account_auth_request(&client, &args.server, &args.secret, &args.device_id).await?;
    let url = args.server.join("/api/v1/account/overview")?;
    let response = client
        .post(url)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<AccountOverview>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        println!("Account: {}", response.public_key_hex);
        println!("Apps: {}", response.app_ids.join(", "));
        println!("Tokens: {}", response.token_count);
        println!("Active: {}", response.active_token_count);
        println!("Revoked: {}", response.revoked_token_count);
        println!("Expired: {}", response.expired_token_count);
    }
    Ok(())
}

async fn account_token_list(args: AccountTokenListArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let auth =
        build_account_auth_request(&client, &args.server, &args.secret, &args.device_id).await?;
    let url = args.server.join("/api/v1/account/tokens/list")?;
    let response = client
        .post(url)
        .json(&TokenListRequest {
            public_key_hex: auth.public_key_hex,
            signature_hex: auth.signature_hex,
            timestamp_ms: auth.timestamp_ms,
            device_id: auth.device_id,
            nonce: auth.nonce,
            app_id_filter: args.app,
        })
        .send()
        .await?
        .error_for_status()?
        .json::<TokenListResponse>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        println!("Account: {}", response.public_key_hex);
        for token in response.tokens {
            println!(
                "- {} | {} | revoked={} | expiresAtMs={}",
                token.token_id, token.app_id, token.revoked, token.expires_at_ms
            );
        }
    }
    Ok(())
}

async fn account_token_revoke(
    args: AccountTokenRevokeArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let request =
        build_account_auth_request(&client, &args.server, &args.secret, &args.device_id).await?;
    let url = args
        .server
        .join(&format!("/api/v1/account/tokens/{}/revoke", args.token_id))?;
    let response = client
        .post(url)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<TokenRevokeResponse>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        println!("Token: {}", response.token_id);
        println!("Revoked: {}", response.revoked);
    }
    Ok(())
}

async fn public_apps(args: PublicAppsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = args.server.join("/api/v1/public/apps")?;
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<AppConfig>>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        for app in response {
            println!(
                "- {} | {} | ttl={}s",
                app.app_id, app.label, app.token_ttl_secs
            );
        }
    }
    Ok(())
}

async fn public_plans(args: PublicPlansArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = args.server.join("/api/v1/public/plans")?;
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<PlanConfig>>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        for plan in response {
            println!(
                "- {} | {} | usd={} | cny={} | maxEntries={}",
                plan.plan_id,
                plan.label,
                plan.yearly_price_usd_cents,
                plan.yearly_price_cny_cents,
                plan.max_entries
            );
        }
    }
    Ok(())
}

async fn developer_meta(args: DeveloperMetaArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = args.server.join("/api/v1/dev/meta")?;
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<DeveloperMetaResponse>()
        .await?;
    if args.json {
        print_json(&response)?;
    } else {
        println!("Developer mode: {}", response.developer_mode);
        println!("Management scope: {}", response.management_scope_app_id);
        println!("Apps: {}", response.apps.len());
        println!("Plans: {}", response.plans.len());
        println!("Accounts: {}", response.stats.account_count);
        println!("App spaces: {}", response.stats.app_space_count);
        println!("Tokens: {}", response.stats.token_count);
    }
    Ok(())
}

fn admin_stats(args: AdminStatsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let store = FileStore::new(&args.data_dir)?;
    let stats = store.store_stats(now_ms())?;
    if args.json {
        print_json(&stats)?;
    } else {
        println!("Accounts: {}", stats.account_count);
        println!("App spaces: {}", stats.app_space_count);
        println!("Tokens: {}", stats.token_count);
        println!("Active: {}", stats.active_token_count);
        println!("Revoked: {}", stats.revoked_token_count);
        println!("Expired: {}", stats.expired_token_count);
    }
    Ok(())
}

async fn fetch_challenge(
    client: &Client,
    server: &Url,
    public_key_hint: &str,
) -> Result<ChallengeResponse, Box<dyn std::error::Error>> {
    let challenge_url = server.join("/api/v1/auth/challenge")?;
    let challenge = client
        .post(challenge_url)
        .json(&serde_json::json!({ "publicKeyHint": public_key_hint }))
        .send()
        .await?
        .error_for_status()?
        .json::<ChallengeResponse>()
        .await?;
    Ok(challenge)
}

fn sign_for_scope(
    secret: &str,
    app_id: &str,
    nonce: &str,
    device_id: &str,
) -> Result<SignedChallenge, Box<dyn std::error::Error>> {
    let timestamp_ms = now_ms();
    let signed = sign_challenge(
        secret,
        &SignatureChallenge {
            app_id: app_id.to_string(),
            nonce: nonce.to_string(),
            timestamp_ms,
            device_id: device_id.to_string(),
        },
    )?;
    Ok(signed)
}

async fn build_account_auth_request(
    client: &Client,
    server: &Url,
    secret: &str,
    device_id: &str,
) -> Result<AccountAuthRequest, Box<dyn std::error::Error>> {
    let identity = derive_identity(secret)?;
    let challenge = fetch_challenge(client, server, &identity.public_key_hex).await?;
    let signed = sign_for_scope(
        secret,
        ACCOUNT_MANAGEMENT_SCOPE,
        &challenge.nonce,
        device_id,
    )?;
    Ok(AccountAuthRequest {
        public_key_hex: signed.public_key_hex,
        signature_hex: signed.signature_hex,
        timestamp_ms: signed.timestamp_ms,
        device_id: signed.device_id,
        nonce: signed.nonce,
    })
}

fn print_json<T: Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
