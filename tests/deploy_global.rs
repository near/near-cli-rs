mod common;
use std::process::Command;

/// An empty but valid Wasm module: magic number and version only.
const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

#[tokio::test]
async fn test_deploy_global_hash_skips_when_already_deployed()
-> Result<(), Box<dyn std::error::Error>> {
    let deployer = near_sandbox::GenesisAccount::default_with_name("deployer.near".parse()?);
    let ctx = common::prepare_tests_with_accounts(vec![deployer.clone()]).await?;
    let wasm_path = ctx.temp_dir.path().join("empty.wasm");
    std::fs::write(&wasm_path, EMPTY_WASM)?;

    let deploy = || {
        Command::new("target/debug/near")
            .env("XDG_CONFIG_HOME", &ctx.config_home)
            .env("HOME", &ctx.config_home)
            .env("APPDATA", &ctx.config_home)
            .args([
                "contract",
                "deploy-as-global",
                "use-file",
                wasm_path.to_str().unwrap(),
                "as-global-hash",
                deployer.account_id.as_str(),
                "network-config",
                "sandbox",
                "sign-with-plaintext-private-key",
                &deployer.private_key,
                "send",
            ])
            .output()
    };

    let first = deploy()?;
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    assert!(
        first.status.success(),
        "first deploy failed: {first_stderr}"
    );
    assert!(
        first_stderr.contains("Global contract has been successfully deployed."),
        "unexpected first deploy output: {first_stderr}"
    );

    let second = deploy()?;
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert!(second.status.success(), "retry failed: {second_stderr}");
    assert!(
        second_stderr.contains("already exists on <sandbox> network. No transaction needed."),
        "retry did not skip: {second_stderr}"
    );
    assert!(
        !second_stderr.contains("successfully deployed"),
        "retry deployed again: {second_stderr}"
    );

    Ok(())
}
