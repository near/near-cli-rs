//! Local JSON-RPC and terminal regressions: never contact a public network.
#[cfg(unix)]
#[test]
fn selected_network_account_checks() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let key =
        near_crypto::SecretKey::from_seed(near_crypto::KeyType::ED25519, "network-scope-test");
    let public_key = key.public_key();
    let delegate = near_primitives::action::delegate::SignedDelegateAction {
        delegate_action: near_primitives::action::delegate::DelegateAction {
            sender_id: "sender.testnet".parse().unwrap(),
            receiver_id: "receiver.testnet".parse().unwrap(),
            actions: vec![],
            nonce: 1,
            max_block_height: 100,
            public_key: public_key.clone(),
        },
        signature: key.sign(b"local test fixture"),
    };
    let fixture = serde_json::json!({
        "secret": key.to_string(),
        "public": public_key.to_string(),
        "implicit": hex::encode(public_key.key_data()),
        "delegate": near_cli_rs::types::signed_delegate_action::SignedDelegateActionAsBase64::from(delegate).to_string(),
    });
    let mut child = Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/selected_network_account_checks.py"
        ))
        .arg(env!("CARGO_BIN_EXE_near"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("Python 3 is required for the local RPC/PTY regression fixture");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(fixture.to_string().as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
}
