use std::process::Command;

use near_cli_rs::commands::transaction::send_signed_transaction::FileSignedTransaction;
use near_cli_rs::config::Config;

#[test]
fn imported_ml_dsa_key_signs_offline_by_full_key_or_handle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = temp_dir.path();
    let config_dir = if cfg!(target_os = "macos") {
        config_home.join("Library/Application Support/near-cli")
    } else {
        config_home.join("near-cli")
    };
    std::fs::create_dir_all(&config_dir).unwrap();
    let mut config = Config {
        credentials_home_dir: config_home.join("credentials"),
        ..Config::default()
    };
    let testnet = config.network_connection.remove("testnet").unwrap();
    config.network_connection.clear();
    config
        .network_connection
        .insert("testnet".to_owned(), testnet);
    config
        .network_connection
        .get_mut("testnet")
        .unwrap()
        .rpc_url = "http://127.0.0.1:1/".parse().unwrap();
    std::fs::create_dir_all(&config.credentials_home_dir).unwrap();
    std::fs::write(config.credentials_home_dir.join("ft_contracts.json"), "[]").unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        toml::to_string(&config.into_latest_version()).unwrap(),
    )
    .unwrap();

    let near = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_near"));
        command
            .current_dir(config_home)
            .env("HOME", config_home)
            .env("XDG_CONFIG_HOME", config_home)
            .env("APPDATA", config_home)
            .arg("--offline");
        command
    };
    let private_key = near_crypto::SecretKey::from_random(near_crypto::KeyType::MLDSA65);
    let public_key = private_key.public_key();
    let handle = near_crypto::PublicKeyHandle::from(&public_key).to_string();
    let imported = near()
        .args([
            "account",
            "import-account",
            "using-private-key",
            &private_key.to_string(),
            "signer.testnet",
            "network-config",
            "testnet",
            "save-to-legacy-keychain",
        ])
        .output()
        .unwrap();
    assert!(imported.status.success(), "offline key import failed");

    for key_id in [public_key.to_string(), handle] {
        let transaction_path = config_home.join("signed-transaction.json");
        let signed = near()
            .args([
                "transaction",
                "construct-transaction",
                "signer.testnet",
                "receiver-id",
                "receiver.testnet",
                "add-action",
                "transfer",
                "1 NEAR",
                "skip",
                "network-config",
                "testnet",
                "sign-with-legacy-keychain",
                "--signer-public-key",
                &key_id,
                "--nonce",
                "1",
                "--block-hash",
                "11111111111111111111111111111111",
                "--block-height",
                "1",
                "save-to-file",
            ])
            .arg(&transaction_path)
            .output()
            .unwrap();
        assert!(
            signed.status.success(),
            "offline signing failed: {}",
            String::from_utf8_lossy(&signed.stderr)
        );
        let signed_transaction: FileSignedTransaction =
            serde_json::from_slice(&std::fs::read(&transaction_path).unwrap()).unwrap();
        let signed_transaction = signed_transaction.signed_transaction;
        assert_eq!(signed_transaction.transaction.public_key(), &public_key);
        assert!(
            signed_transaction.signature.verify(
                signed_transaction
                    .transaction
                    .get_hash_and_size()
                    .0
                    .as_ref(),
                &public_key,
            )
        );
        std::fs::remove_file(transaction_path).unwrap();
    }
}
