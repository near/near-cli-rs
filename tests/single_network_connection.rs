use std::process::{Command, Stdio};

// With exactly one connection, near skips the network prompt. It used to
// answer with the connection's `network_name` instead of the connection
// name, so commands panicked ("Failed to get network config!") whenever the
// two differed.
#[test]
fn single_connection_is_selected_by_its_connection_name() {
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
    // Port 9 is closed, so the RPC request fails at once.
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            r#"version = "5"
credentials_home_dir = "{}"

[network_connection.my-sandbox]
network_name = "sandbox"
rpc_url = "http://127.0.0.1:9/"
wallet_url = "http://127.0.0.1:9/"
explorer_transaction_url = "http://127.0.0.1:9/transactions/"
"#,
            credentials_dir.to_string_lossy().replace('\\', "\\\\"),
        ),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_near"))
        .args([
            "transaction",
            "view-status",
            "9wrX7i3jZgqUsyXLH3ja4Pyu9UFE2W6HFabq9raBAQMT",
        ])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env("APPDATA", home.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(101), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    // It gets as far as the RPC call on the selected connection.
    assert!(stderr.contains("127.0.0.1:9"), "{stderr}");
}
