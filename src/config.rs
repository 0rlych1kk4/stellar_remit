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
        // Load from .env if present
        dotenv().ok();

        // Build configuration from optional file and environment variables
        let builder = Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            // Use the STELLAR_ prefix without any additional separator.
            // After stripping "STELLAR_", keys like "HORIZON_URL" map to "horizon_url"
            .add_source(config::Environment::with_prefix("STELLAR"));

        let settings = builder.build().context("Failed to build configuration")?;

        let cfg = settings
            .try_deserialize::<AppConfig>()
            .context("Failed to deserialize configuration")?;

        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_init() {
        // Set exactly the STELLAR_… vars that AppConfig expects
        std::env::set_var("STELLAR_SENDER_SECRET", "SXXXXXXXXXXXXXXXXXXXXXXXXXXX");
        std::env::set_var("STELLAR_RECEIVER_ADDRESS", "GXXXXXXXXXXXXXXXXXXXXXXXXXXX");
        std::env::set_var("STELLAR_HORIZON_URL", "https://horizon-testnet.stellar.org");

        let cfg = AppConfig::init().expect("Should load config");

        assert!(cfg.sender_secret.starts_with('S'));
        assert!(cfg.receiver_address.starts_with('G'));
        assert!(cfg.horizon_url.contains("https://"));
    }
}
