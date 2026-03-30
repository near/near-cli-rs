use near_sandbox::{Sandbox, SandboxConfig};

pub struct TestContext {
    #[allow(dead_code)]
    pub sandbox: Sandbox,
    #[allow(dead_code)]
    pub temp_dir: tempfile::TempDir,
    pub config_home: std::path::PathBuf,
}

pub async fn prepare_tests() -> Result<TestContext, Box<dyn std::error::Error>> {
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
    let config_home = temp_dir.path().to_path_buf();

    // Place config where dirs::config_dir() resolves for the child process:
    //   Linux:   $XDG_CONFIG_HOME/near-cli  (we set XDG_CONFIG_HOME=config_home)
    //   macOS:   $HOME/Library/Application Support/near-cli  (we set HOME=config_home)
    //   Windows: $APPDATA/near-cli  (we set APPDATA=config_home)
    let near_cli_config_dir = if cfg!(target_os = "macos") {
        config_home.join("Library/Application Support/near-cli")
    } else {
        config_home.join("near-cli")
    };
    std::fs::create_dir_all(&near_cli_config_dir)?;
    let config_path = near_cli_config_dir.join("config.toml");

    // Write a config file pointing to our sandbox
    let credentials_dir = temp_dir.path().join("credentials");
    std::fs::create_dir_all(&credentials_dir)?;

    // Pre-create ft_contracts.json to avoid blocking HTTP calls to nearblocks API at startup
    std::fs::write(credentials_dir.join("ft_contracts.json"), "[]")?;

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

    Ok(TestContext {
        sandbox,
        temp_dir,
        config_home,
    })
}
