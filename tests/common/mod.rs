use near_sandbox::Sandbox;

pub async fn prepare_tests() -> Result<(Sandbox, tempfile::TempDir), Box<dyn std::error::Error>> {
    // Start a local sandbox
    let sandbox = Sandbox::start_sandbox().await?;

    // Create a temporary config directory for this test
    let temp_dir = tempfile::tempdir()?;

    // Create the config directory structure that near-cli expects:
    // $XDG_CONFIG_HOME/near-cli/config.toml
    let config_home = temp_dir.path();
    let near_cli_config_dir = config_home.join("near-cli");
    std::fs::create_dir_all(&near_cli_config_dir)?;
    let config_path = near_cli_config_dir.join("config.toml");

    std::env::set_var("XDG_CONFIG_HOME", config_home); // Linux
    std::env::set_var("HOME", config_home); // macOS
    std::env::set_var("APPDATA", config_home); // Windows

    // Write a config file pointing to our sandbox
    let credentials_dir = temp_dir.path().join("credentials");
    std::fs::create_dir_all(&credentials_dir)?;

    // Format the RPC URL properly
    let rpc_url = format!("{}/", sandbox.rpc_addr);

    // Write a V3 config to avoid migration issues
    let config_content = format!(
        r#"version = "3"
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
