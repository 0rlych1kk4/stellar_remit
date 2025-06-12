use anyhow::{anyhow, Context, Result};
use config::AppConfig;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Load configuration
    let cfg = AppConfig::init()?;

    // Build keypairs
    let sender_kp = SodiumKeyPair::from_secret_seed(&cfg.sender_secret)
        .context("Invalid SENDER_SECRET seed")?;
    let receiver_pk = PublicKey::from_account_id(&cfg.receiver_address)
        .context("Invalid RECEIVER_ADDRESS key")?;

    let http = Client::new();

    // Retry GET /accounts on server error
    let acct_url = format!("{}/accounts/{}", cfg.horizon_url, sender_kp.public_key());
    let acct_text = {
        let mut attempts = 0;
        loop {
            let resp = http
                .get(&acct_url)
                .send()
                .await
                .context("Failed to GET account info")?;
            let status = resp.status();
            let body = resp.text().await.context("Failed to read account response")?;
            if status.is_success() {
                break body;
            } else if status.is_server_error() && attempts < 2 {
                attempts += 1;
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

    // Build the payment operation
    let stm = Stroops::new(1_000_000);
    let payment_op = Operation::new_payment()
        .with_destination(receiver_pk)
        .with_asset(Asset::new_native())
        .with_amount(stm)
        .context("Invalid amount")?
        .build()
        .context("Failed to build payment operation")?;

    // Build & sign transaction
    let mut tx = Transaction::builder(sender_kp.public_key(), seq + 1, MIN_BASE_FEE)
        .add_operation(payment_op)
        .with_memo(Memo::Text("Remittance".into()))
        .into_transaction()
        .context("Failed to build transaction")?;
    tx.sign(&sender_kp.as_ref(), &Network::new_test())
        .context("Failed to sign transaction")?;

    // Serialize envelope to XDR
    let envelope = tx.into_envelope();
    let envelope_xdr = envelope
        .xdr_base64()
        .context("Failed to serialize envelope to base64 XDR")?;

    // Submit via HTTP POST
    let submit_url = format!("{}/transactions", cfg.horizon_url);
    let resp = http
        .post(&submit_url)
        .form(&[("tx", envelope_xdr)])
        .send()
        .await
        .context("Failed to POST transaction")?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .context("Failed to read submit response")?;

    if !status.is_success() {
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            if let Some(code) = json["extras"]["result_codes"]["transaction"].as_str() {
                let msg = match code {
                    "tx_bad_seq" => "Sequence error: please retry after refreshing sequence.",
                    "tx_insufficient_balance" => "Insufficient balance: top up your account.",
                    other => return Err(anyhow!("Transaction failed with code: {}", other)),
                };
                return Err(anyhow!(msg));
            }
        }
        return Err(anyhow!("Horizon error submitting tx ({}): {}", status, text));
    }

    // Parse and display the transaction hash
    let json: Value = serde_json::from_str(&text).context("Invalid JSON from submit")?;
    let hash = json["hash"].as_str().ok_or_else(|| anyhow!("No `hash` in response"))?;
    println!("Transaction sent! Hash: {}", hash);

    Ok(())
}

