use std::env;
use std::str::FromStr;

use reqwest::Client;
use serde::Deserialize;
use stellar_base::{
    amount::{Amount, Stroops},
    asset::Asset,
    crypto::{KeyPair, SecretKey},
    memo::Memo,
    network::Network,
    operations::{Operation, OperationBody},
    public_key::PublicKey,
    transaction::TransactionBuilder,
    xdr::XDRSerialize,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let sender_secret = env::var("SENDER_SECRET")?;
    let receiver_address = env::var("RECEIVER_ADDRESS")?;

    let sender_secret_key = SecretKey::from_encoding(&sender_secret)?;
    let sender_keypair = KeyPair::new(
        sender_secret_key.clone(),
        sender_secret_key.get_public(),
    );

    let sender_pub = sender_keypair.public_key();
    let receiver_pub = PublicKey::from_encoding(&receiver_address)?;

    let sequence_number = get_sequence_number(&sender_pub.to_encoding()).await?;

    let amount = Amount::from_str("10.0")?;

    // Build payment operation manually
    let payment_operation = Operation {
        source_account: None,
        body: OperationBody::Payment {
            destination: receiver_pub.clone().into(),
            asset: Asset::new_native(),
            amount,
        },
    };

    let network = Network::new_test();

    // The fee here should be in Stroops
    let fee = Stroops(100);

    let mut tx = TransactionBuilder::new(sender_pub.clone().into(), sequence_number, fee)
        .add_operation(payment_operation)
        .with_memo(Memo::None)
        .into_transaction()?;

    tx.sign(&sender_keypair, &network)?;

    let tx_xdr = tx.to_envelope().xdr_base64()?;
    println!("Built transaction XDR:\n{}", tx_xdr);

    Ok(())
}

#[derive(Deserialize)]
struct AccountResponse {
    sequence: String,
}

async fn get_sequence_number(account_id: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let url = format!("https://horizon-testnet.stellar.org/accounts/{}", account_id);
    let client = Client::new();
    let res = client.get(&url).send().await?;
    let account: AccountResponse = res.json().await?;
    Ok(account.sequence.parse::<i64>()?)
}

