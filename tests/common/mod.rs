use near_sandbox::{Sandbox, SandboxConfig};

pub async fn prepare_tests() -> Result<(Sandbox, tempfile::TempDir), Box<dyn std::error::Error>> {
    // Configure the sandbox with a custom epoch length
    let config = SandboxConfig {
        additional_genesis: Some(serde_json::json!({
            "epoch_length": 43200,
        })),
        ..Default::default()
    };
    // Start a local sandbox
    let sandbox = Sandbox::start_sandbox_with_config(config).await?;

    // Create a temporary config directory for this test
    let temp_dir = tempfile::tempdir()?;

    // Create the config directory structure that near-cli expects:
    // $XDG_CONFIG_HOME/near-cli/config.toml
    let config_home = temp_dir.path();

    std::env::set_var("XDG_CONFIG_HOME", config_home); // Linux
    std::env::set_var("HOME", config_home); // macOS
    std::env::set_var("APPDATA", config_home); // Windows

    let near_cli_config_dir = dirs::config_dir().unwrap().join("near-cli");
    std::fs::create_dir_all(&near_cli_config_dir)?;
    let config_path = near_cli_config_dir.join("config.toml");

    // Write a config file pointing to our sandbox
    let credentials_dir = temp_dir.path().join("credentials");
    std::fs::create_dir_all(&credentials_dir)?;

    // Format the RPC URL properly
    let rpc_url = format!("{}/", sandbox.rpc_addr);

    // Write a V4 config to avoid migration issues
    let config_content = format!(
        r#"version = "4"
credentials_home_dir = "{}"

[network_connection.sandbox]
network_name = "sandbox"
rpc_url = "{}"
wallet_url = "{}"
explorer_transaction_url = "{}transactions/"
"#,
        credentials_dir.to_string_lossy(),
        rpc_url,
        rpc_url,
        rpc_url
    );
    std::fs::write(&config_path, config_content)?;

    Ok((sandbox, temp_dir))
}
