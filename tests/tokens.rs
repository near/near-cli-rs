mod common;
use std::process::Command;

#[tokio::test]
async fn test_view_near_balance() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = common::prepare_tests().await?;

    let output = Command::new("target/debug/near")
        .env("XDG_CONFIG_HOME", &ctx.config_home)
        .env("HOME", &ctx.config_home)
        .env("APPDATA", &ctx.config_home)
        .args([
            "tokens",
            "test.near",
            "view-near-balance",
            "network-config",
            "sandbox",
            "now",
        ])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let normalize_output = stderr.to_string();

    insta::assert_snapshot!("view_near_balance", normalize_output);

    Ok(())
}
