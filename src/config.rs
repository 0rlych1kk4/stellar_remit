use dotenvy::dotenv;
use std::env;

pub struct AppConfig {
    pub horizon_url: String,
    pub sender_secret: String,
}

impl AppConfig {
    pub fn init() -> Self {
        dotenv().ok();
        Self {
            horizon_url: env::var("STELLAR_HORIZON").expect("HORIZON URL not set"),
            sender_secret: env::var("SENDER_SECRET").expect("Sender secret not set"),
        }
    }
}

