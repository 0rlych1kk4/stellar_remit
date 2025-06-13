use anyhow::{anyhow, Context, Result};
use clap::Parser;
use config::AppConfig;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber;

use stellar_base::{
    amount::Stroops,
    asset::Asset,
    crypto::{SodiumKeyPair, PublicKey},
    memo::Memo,
    network::Network,
    operations::Operation,
    transaction::{Transaction, MIN_BASE_FEE},
    xdr::XDRSerialize,
};

mod config;

/// CLI options, which override env vars when present
#[derive(Parser)]
#[command(name = "stellar-remit")]
struct Cli {
    /// Amount in stroops (1 XLM = 1_000_000 stroops)
    #[arg(long)]
    amount: Option<i64>,

    /// Recipient address (G... public key)
    #[arg(long)]
    to: Option<String>,

    /// Memo text
    #[arg(long)]
    memo: Option<String>,

    /// Horizon URL override
    #[arg(long)]
    horizon: Option<String>,

    /// Sender secret override
    #[arg(long)]
    secret: Option<String>,

    /// Receiver address override
    #[arg(long)]
    receiver: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init(); // Initialize global subscriber

    let args = Cli::parse();
    if let Err(e) = run(args).await {
        error!("{:#}", e);
        std::process::exit(1);
    }
}

async fn run(args: Cli) -> Result<()> {
    info!("Initializing configuration...");
    let mut cfg = AppConfig::init()?;

    if let Some(h) = args.horizon {
        cfg.horizon_url = h;
    }
    if let Some(s) = args.secret {
        cfg.sender_secret = s;
    }
    if let Some(r) = args.receiver {
        cfg.receiver_address = r;
    }

    let sender_kp = SodiumKeyPair::from_secret_seed(&cfg.sender_secret)
        .context("Invalid SENDER_SECRET seed")?;
    let receiver_pk = PublicKey::from_account_id(&cfg.receiver_address)
        .context("Invalid RECEIVER_ADDRESS key")?;

    let stm = Stroops::new(args.amount.unwrap_or(1_000_000));
    let memo_text = args.memo.unwrap_or_else(|| "Remittance".into());

    let http = Client::new();
    let acct_url = format!("{}/accounts/{}", cfg.horizon_url, sender_kp.public_key());

    info!("Fetching account info from Horizon...");
    let acct_text = {
        let mut attempts = 0;
        loop {
            let resp = http.get(&acct_url).send().await.context("GET /accounts failed")?;
            let status = resp.status();
            let body = resp.text().await.context("Read account body failed")?;

            if status.is_success() {
                break body;
            } else if status.is_server_error() && attempts < 2 {
                attempts += 1;
                info!("Retrying Horizon request... attempt {}", attempts);
                sleep(Duration::from_millis(500 * attempts)).await;
                continue;
            } else {
                return Err(anyhow!("Horizon error fetching account: {}", body));
            }
        }
    };

    let acct_json: Value = serde_json::from_str(&acct_text)
        .context("Failed to parse account JSON")?;

    let seq: i64 = acct_json["sequence"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow!("Invalid sequence in account JSON"))?;

    let payment_op = Operation::new_payment()
        .with_destination(receiver_pk)
        .with_asset(Asset::new_native())
        .with_amount(stm)
        .context("Invalid amount")?
        .build()
        .context("Build payment operation failed")?;

    let mut tx = Transaction::builder(sender_kp.public_key(), seq + 1, MIN_BASE_FEE)
        .add_operation(payment_op)
        .with_memo(Memo::Text(memo_text))
        .into_transaction()
        .context("Build transaction failed")?;
    tx.sign(&sender_kp.as_ref(), &Network::new_test())
        .context("Sign transaction failed")?;

    let envelope = tx.into_envelope();
    let envelope_xdr = envelope.xdr_base64().context("Serialize XDR failed")?;

    info!("Submitting transaction...");
    let submit_url = format!("{}/transactions", cfg.horizon_url);
    let resp = http
        .post(&submit_url)
        .form(&[("tx", envelope_xdr)])
        .send()
        .await
        .context("POST transaction failed")?;
    let status = resp.status();
    let text = resp.text().await.context("Read submit response failed")?;

    if !status.is_success() {
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            if let Some(code) = json["extras"]["result_codes"]["transaction"].as_str() {
                let friendly = match code {
                    "tx_bad_seq" => "Sequence error: please retry with updated sequence.",
                    "tx_insufficient_balance" => "Insufficient balance: please fund your account.",
                    other => return Err(anyhow!("Transaction failed with code: {}", other)),
                };
                return Err(anyhow!(friendly));
            }
        }
        return Err(anyhow!("Horizon error ({}): {}", status, text));
    }

    let json: Value = serde_json::from_str(&text).context("Parse submit response JSON failed")?;
    let hash = json["hash"]
        .as_str()
        .ok_or_else(|| anyhow!("No hash in response"))?;

    info!("Transaction successful! Hash: {}", hash);
    Ok(())
}

