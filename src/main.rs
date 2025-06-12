use anyhow::{anyhow, Context, Result};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
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
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // 1) Load environment variables
    let horizon_url   = env::var("STELLAR_HORIZON")
        .context("STELLAR_HORIZON env var not set")?;
    let sender_secret = env::var("SENDER_SECRET")
        .context("SENDER_SECRET env var not set")?;
    let receiver_addr = env::var("RECEIVER_ADDRESS")
        .context("RECEIVER_ADDRESS env var not set")?;

    // 2) Build keypairs
    let sender_kp = SodiumKeyPair::from_secret_seed(&sender_secret)
        .context("Invalid SENDER_SECRET seed")?;
    let receiver_pk = PublicKey::from_account_id(&receiver_addr)
        .context("Invalid RECEIVER_ADDRESS key")?;

    // 3) Fetch current sequence
    let http = Client::new();
    let acct_url = format!("{}/accounts/{}", horizon_url, sender_kp.public_key());
    let acct_res = http
        .get(&acct_url)
        .send()
        .await
        .context("Failed to GET account info")?;
    let status_code = acct_res.status();
    let acct_text = acct_res
        .text()
        .await
        .context("Failed to read account response")?;
    if !status_code.is_success() {
        return Err(anyhow!("Horizon error fetching account: {}", acct_text));
    }
    let acct_json: Value = serde_json::from_str(&acct_text)
        .context("Failed to parse account JSON")?;
    let seq: i64 = acct_json["sequence"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow!("Invalid sequence in account JSON"))?;

    // 4) Build the payment operation
    let stm = Stroops::new(1_000_000); // 1 XLM = 1,000,000 stroops
    let payment_op = Operation::new_payment()
        .with_destination(receiver_pk)
        .with_asset(Asset::new_native())
        .with_amount(stm)
        .context("Invalid amount")?
        .build()
        .context("Failed to build payment operation")?;

    // 5) Build & sign transaction
    let mut tx = Transaction::builder(sender_kp.public_key(), seq + 1, MIN_BASE_FEE)
        .add_operation(payment_op)
        .with_memo(Memo::Text("Remittance".into()))
        .into_transaction()
        .context("Failed to build transaction")?;
    tx.sign(&sender_kp.as_ref(), &Network::new_test())
        .context("Failed to sign transaction")?;

    // 6) Serialize envelope to XDR
    let envelope = tx.into_envelope();
    let envelope_xdr = envelope
        .xdr_base64()
        .context("Failed to serialize envelope to base64 XDR")?;

    // 7) Submit via HTTP POST
    let submit_url = format!("{}/transactions", horizon_url);
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
        return Err(anyhow!("Horizon error submitting tx ({}): {}", status, text));
    }

    // 8) Parse and display the transaction hash
    let json: Value = serde_json::from_str(&text)
        .context("Invalid JSON from submit")?;
    let hash = json["hash"]
        .as_str()
        .ok_or_else(|| anyhow!("No `hash` in response"))?;
    println!("Transaction sent! Hash: {}", hash);

    Ok(())
}

