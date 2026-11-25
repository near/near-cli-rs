use std::process::{Command, Stdio};

// `at-block-hash` used to unwrap the parsed hash, so a malformed hash
// panicked (exit code 101) instead of being rejected as invalid input.
#[test]
fn malformed_block_hash_is_rejected_without_panicking() {
    let home = tempfile::tempdir().unwrap();
    let credentials_dir = home.path().join("credentials");
    std::fs::create_dir_all(&credentials_dir).unwrap();
    std::fs::write(
        credentials_dir.join("ft_contracts.json"),
        r#"[{"symbol":"wNEAR","decimals":24,"contract":"wrap.near"}]"#,
    )
    .unwrap();
    let config_dir = if cfg!(target_os = "macos") {
        home.path().join("Library/Application Support/near-cli")
    } else {
        home.path().join("near-cli")
    };
    std::fs::create_dir_all(&config_dir).unwrap();
    // Port 9 is closed, so a request that slips through fails at once.
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            r#"version = "5"
credentials_home_dir = "{}"

[network_connection.sandbox]
network_name = "sandbox"
rpc_url = "http://127.0.0.1:9/"
wallet_url = "http://127.0.0.1:9/"
explorer_transaction_url = "http://127.0.0.1:9/transactions/"
"#,
            credentials_dir.to_string_lossy().replace('\\', "\\\\"),
        ),
    )
    .unwrap();

    for hash in [
        "x",
        "679",
        "u2.tyb",
        "11111111111111111111111111111111111111111111111",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_near"))
            .args([
                "account",
                "view-account-summary",
                "alice.near",
                "network-config",
                "sandbox",
                "at-block-hash",
                hash,
            ])
            .env("HOME", home.path())
            .env("XDG_CONFIG_HOME", home.path())
            .env("APPDATA", home.path())
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_ne!(output.status.code(), Some(101), "{hash}: {stderr}");
        assert!(!output.status.success(), "{hash}: {stderr}");
        assert!(!stderr.contains("panicked"), "{hash}: {stderr}");
    }
}
