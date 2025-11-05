mod common;
use common::prepare_tests;
use std::process::Command;

#[tokio::test]
async fn test_view_near_balance() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = prepare_tests().await?;

    let output = Command::new("target/debug/near")
        .args(&[
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