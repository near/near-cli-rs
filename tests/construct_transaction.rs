mod common;
use std::process::Command;

const DUMMY_PUBLIC_KEY: &str = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp";
const DUMMY_BLOCK_HASH: &str = "11111111111111111111111111111111";

fn sign_later_args() -> Vec<&'static str> {
    vec![
        "network-config",
        "sandbox",
        "sign-later",
        "--signer-public-key",
        DUMMY_PUBLIC_KEY,
        "--nonce",
        "0",
        "--block-hash",
        DUMMY_BLOCK_HASH,
        "display",
    ]
}

/// Replace the binary path in output with a stable placeholder
fn normalize_output(output: &str) -> String {
    let re = regex::Regex::new(r"\$ .+ transaction sign-transaction").unwrap();
    re.replace_all(output, "$ near transaction sign-transaction")
        .to_string()
}

#[tokio::test]
async fn test_construct_transaction_old_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = common::prepare_tests().await?;

    let mut args = vec![
        "transaction",
        "construct-transaction",
        "test.near",
        "test.near",
        "add-action",
        "transfer",
        "1 NEAR",
        "skip",
    ];
    args.extend(sign_later_args());

    let output = Command::new("target/debug/near").args(&args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Command failed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    insta::assert_snapshot!("construct_transaction_old_syntax", normalize_output(&stdout));

    Ok(())
}

#[tokio::test]
async fn test_construct_transaction_new_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = common::prepare_tests().await?;

    let mut args = vec![
        "transaction",
        "construct-transaction",
        "test.near",
        "account-id",
        "test.near",
        "add-action",
        "transfer",
        "1 NEAR",
        "skip",
    ];
    args.extend(sign_later_args());

    let output = Command::new("target/debug/near").args(&args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Command failed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Old syntax and new syntax should produce identical output
    insta::assert_snapshot!("construct_transaction_new_syntax", normalize_output(&stdout));

    Ok(())
}

#[tokio::test]
async fn test_construct_transaction_state_init() -> Result<(), Box<dyn std::error::Error>> {
    let (_sandbox, _temp_dir) = common::prepare_tests().await?;

    // A valid base58-encoded CryptoHash (32 zero bytes in base58)
    let hash = DUMMY_BLOCK_HASH;

    let mut args = vec![
        "transaction",
        "construct-transaction",
        "test.near",
        "state-init",
        "use-global-hash",
        hash,
        "data-from-json",
        "{}",
        "deposit",
        "0 NEAR",
        "skip",
    ];
    args.extend(sign_later_args());

    let output = Command::new("target/debug/near").args(&args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Command failed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    insta::assert_snapshot!("construct_transaction_state_init", normalize_output(&stdout));

    Ok(())
}
