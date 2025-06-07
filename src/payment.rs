use crate::errors::AppError;
use reqwest::Client;
use stellar_sdk::{
    asset::Asset,
    keypair::Keypair,
    network::Network,
    transaction::Transaction,
};

pub async fn send_payment(
    client: &Client,
    horizon_url: &str,
    sender_secret: &str,
    recipient_address: &str,
    amount: &str,
) -> Result<String, AppError> {
    let sender = Keypair::from_secret(sender_secret)?;
    let account_url = format!("{}/accounts/{}", horizon_url, sender.public_key());
    let account_info: serde_json::Value = client.get(&account_url).send().await?.json().await?;
    let sequence = account_info["sequence"].as_str().unwrap().parse::<i64>()?;

    let tx = Transaction::builder(sender.public_key(), sequence)
        .add_operation(
            stellar_sdk::operation::Payment::new(
                recipient_address.parse()?,
                Asset::native(),
                amount.parse()?,
            )?
        )
        .into_transaction()?
        .sign(&sender, &Network::new_test())?;

    let envelope_xdr = tx.to_xdr_base64()?;
    let res: serde_json::Value = client
        .post(format!("{}/transactions", horizon_url))
        .form(&[("tx", envelope_xdr)])
        .send()
        .await?
        .json()
        .await?;

    Ok(res.to_string())
}

