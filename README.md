# Stellar Remit

[![CI](https://github.com/0rlych1kk4/stellar_remit/actions/workflows/ci.yml/badge.svg)](https://github.com/0rlych1kk4/stellar_remit/actions/workflows/ci.yml)

A simple CLI and HTTP health/metrics server for sending payments on the Stellar network.

## Features

- **CLI Remittance**: Send XLM payments via Horizon Testnet/Mainnet.
- **Structured Logging**: Using [tracing](https://crates.io/crates/tracing).
- **Automatic Retries**: Exponential backoff for account fetch failures.
- **Health & Metrics**: Lightweight Axum server exposing `/health` and `/metrics`.
- **Configurable**: Load settings from `.env`, `config/default.toml`, or environment.
- **CI Pipeline**: GitHub Actions for formatting, linting, and testing.

## Installation

Requires Rust 1.65+.

git clone https://github.com/0rlych1kk4/stellar_remit.git
cd stellar_remit
cargo build --release

You can also install via cargo install --path . to get stellar-remit on your $PATH.

## Configuration

Create a .env (or use environment) with the following variables:

STELLAR_SENDER_SECRET=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
STELLAR_RECEIVER_ADDRESS=GYYYYYYYYYYYYYYYYYYYYYYYYYYY
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
Optionally, you can supply a config/default.toml:

horizon_url = "https://horizon-testnet.stellar.org"
sender_secret = "SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
receiver_address = "GYYYYYYYYYYYYYYYYYYYYYYYYYYY"
Environment variables override any file settings.

## Usage

CLI
stellar-remit \
  --amount 5000000 \
  --to GDESTINATIONADDRESS... \
  --memo "Rent payment"
Flags:

--amount (optional): Stroops (1 XLM = 1 000 000 stroops). Defaults to 1 000 000 (1 XLM).
--to (optional): Override receiver_address from config.
--memo (optional): Transaction memo; defaults to "Remittance".
--horizon, --secret, --receiver: override corresponding config entries.
Health & Metrics Server
Runs on port 3000 by default, in background when you start the CLI.

GET /health → 200 OK, body OK
GET /metrics → Prometheus‐style metrics (dummy gauge for now)
Testing

Unit & Integration
cargo test
Format Check
cargo fmt -- --check
Lint with Clippy
cargo clippy --all-targets -- -D warnings
Continuous Integration

See .github/workflows/ci.yml for details:

Runs on every push/pull_request
Checks formatting, lints, and runs the full test suite

## Contributing

Fork the repo
Create your feature branch:
git checkout -b feature/your-feature
Commit your changes with descriptive message
and add Co-authored-by: tags as appropriate
Push to the branch and open a pull request
Please run cargo fmt and cargo clippy before submitting.

## License

This project is licensed under the MIT License. See LICENSE for details.
