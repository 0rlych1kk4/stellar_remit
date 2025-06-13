use anyhow::Context;
use config::Config;
use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub horizon_url: String,
    pub sender_secret: String,
    pub receiver_address: String,
}

impl AppConfig {
    pub fn init() -> anyhow::Result<Self> {
        // Load from .env file if it exists
        dotenv().ok();

        // Build configuration from file and environment
        let builder = Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("STELLAR").separator("_"));

        // Deserialize into AppConfig struct
        let cfg = builder.build()?.try_deserialize::<AppConfig>()
            .context("Failed to deserialize configuration")?;

        Ok(cfg)
    }
}

