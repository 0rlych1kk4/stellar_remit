use anyhow::Context;
use dotenvy::dotenv;
use std::env;

pub struct AppConfig {
    pub horizon_url: String,
    pub sender_secret: String,
    pub receiver_address: String,
}

impl AppConfig {
    pub fn init() -> anyhow::Result<Self> {
        dotenv().ok();

        let horizon_url = env::var("STELLAR_HORIZON")
            .context("STELLAR_HORIZON env var not set")?;
        let sender_secret = env::var("SENDER_SECRET")
            .context("SENDER_SECRET env var not set")?;
        let receiver_address = env::var("RECEIVER_ADDRESS")
            .context("RECEIVER_ADDRESS env var not set")?;

        Ok(Self {
            horizon_url,
            sender_secret,
            receiver_address,
        })
    }
}

