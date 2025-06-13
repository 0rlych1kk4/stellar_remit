use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_config_init_loads_env_or_toml() {
    // Write a temporary .env.test with the STELLAR_… variables
    let env_path = Path::new(".env.test");
    fs::write(
        env_path,
        r#"
STELLAR_SENDER_SECRET=SXXXXXXXXXXXXXXXXXXXXXXXXXXX
STELLAR_RECEIVER_ADDRESS=GXXXXXXXXXXXXXXXXXXXXXXXX
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
"#,
    )
    .unwrap();

    // Load that file into the environment
    dotenvy::from_path(env_path).ok();

    // Act
    let cfg = stellar_remit::config::AppConfig::init().expect("Should load config");

    // Assert
    assert!(cfg.sender_secret.starts_with('S'));
    assert!(cfg.receiver_address.starts_with('G'));
    assert!(cfg.horizon_url.contains("horizon"));

    // Cleanup
    let _ = fs::remove_file(env_path);
}
