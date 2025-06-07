use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("HTTP error")]
    Http(#[from] reqwest::Error),

    #[error("Stellar error")]
    Stellar(#[from] stellar_sdk::Error),

    #[error("Other: {0}")]
    Other(String),
}

