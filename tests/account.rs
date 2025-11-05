mod common;
use std::process::Command;

#[tokio::test]
async fn test_view_account_summary_with_localnet() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = common::prepare_tests().await?;

    let output = Command::new("target/debug/near")
        .args(&[
            "account",
            "view-account-summary",
            "test.near",
            "network-config",
            "sandbox",
            "now",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let normalized_stdout = normalize_output(&stdout);

    insta::assert_snapshot!("view_account_summary", normalized_stdout);

    Ok(())
}

#[tokio::test]
async fn test_view_account_summary_nonexistent() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = common::prepare_tests().await?;

    let output = Command::new("target/debug/near")
        .args(&[
            "account",
            "view-account-summary",
            "nonexistent.near",
            "network-config",
            "sandbox",
            "now",
        ])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("account nonexistent.near does not exist while viewing"));

    Ok(())
}

/// Normalize output by replacing dynamic content with placeholders
fn normalize_output(output: &str) -> String {
    use regex::Regex;

    let normalized = output;

    // Replace block numbers (e.g., "At block #19" -> "At block #[BLOCK_NUM]")
    let block_regex = Regex::new(r"At block #\d+").unwrap();
    let normalized = block_regex.replace_all(&normalized, "At block #[BLOCK_NUM]");

    // Replace block hashes (e.g., "(Gqo3Sym99tdtKm9Ha2aVFUvPPcqNVX8qfQ3dvpUk9B51)" -> "([BLOCK_HASH])")
    let hash_regex = Regex::new(r"\([A-Za-z0-9]{44}\)").unwrap();
    let normalized = hash_regex.replace_all(&normalized, "([BLOCK_HASH])");

    normalized.to_string()
}
