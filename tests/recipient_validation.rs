use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use near_cli_rs::config::Config;
use serde_json::{Value, json};

// Only local read-only RPC queries are accepted; no keys or transactions are submitted.
struct Fixture {
    home: tempfile::TempDir,
    requests: Arc<Mutex<Vec<Value>>>,
    stop: mpsc::Sender<()>,
    server: Option<std::thread::JoinHandle<()>>,
}

impl Fixture {
    fn new(receiver_exists: bool) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = mpsc::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = requests.clone();
        let server = std::thread::spawn(move || {
            while matches!(stopped.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(err) => panic!("accept: {err}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(&mut stream);
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((name, value)) = line.split_once(':')
                        && name.eq_ignore_ascii_case("content-length")
                    {
                        length = value.trim().parse::<usize>().unwrap();
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                let request: Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(request["method"], "query", "unexpected RPC: {request}");
                let params = &request["params"];
                let hash = near_primitives::hash::CryptoHash::default();
                let mut response = match params["request_type"].as_str().unwrap() {
                    "view_account" if !receiver_exists && params["account_id"] != "ft.near" => {
                        json!({"error": {
                            "code": -32000, "message": "Server error",
                            "data": "account does not exist", "name": "HANDLER_ERROR",
                            "cause": {"name": "UNKNOWN_ACCOUNT", "info": {
                                "requested_account_id": params["account_id"],
                                "block_height": 1, "block_hash": hash
                            }}
                        }})
                    }
                    "view_account" => json!({"result": {
                        "amount": "1000000000000000000000000", "locked": "0",
                        "storage_usage": 0, "code_hash": hash,
                        "block_height": 1, "block_hash": hash
                    }}),
                    "call_function" => {
                        let result = match params["method_name"].as_str().unwrap() {
                            "ft_metadata" => json!({"symbol": "TEST", "decimals": 0}),
                            "storage_balance_of" => json!({"total": "1", "available": "0"}),
                            method => panic!("unexpected view method: {method}"),
                        };
                        json!({"result": {
                            "result": serde_json::to_vec(&result).unwrap(), "logs": [],
                            "block_height": 1, "block_hash": hash
                        }})
                    }
                    other => panic!("unexpected query: {other}"),
                };
                response["jsonrpc"] = json!("2.0");
                response["id"] = request["id"].clone();
                server_requests.lock().unwrap().push(request);
                let response = response.to_string();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
            }
        });
        let home = tempfile::tempdir().unwrap();
        let config_dir = if cfg!(target_os = "macos") {
            home.path().join("Library/Application Support/near-cli")
        } else {
            home.path().join("near-cli")
        };
        std::fs::create_dir_all(&config_dir).unwrap();
        let mut config = Config {
            credentials_home_dir: home.path().join("credentials"),
            ..Config::default()
        };
        let mut network = config.network_connection.remove("mainnet").unwrap();
        network.rpc_url = format!("http://{address}/").parse().unwrap();
        network.rpc_api_key = None;
        config.network_connection.clear();
        config
            .network_connection
            .insert("mainnet".to_owned(), network);
        std::fs::create_dir_all(&config.credentials_home_dir).unwrap();
        std::fs::write(
            config.credentials_home_dir.join("ft_contracts.json"),
            r#"[{"contract":"ft.near","symbol":"TEST","decimals":0}]"#,
        )
        .unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            toml::to_string(&config.into_latest_version()).unwrap(),
        )
        .unwrap();
        Self {
            home,
            requests,
            stop,
            server: Some(server),
        }
    }

    fn run(
        &self,
        transfer: &str,
        receiver: &str,
        offline: bool,
        quiet: bool,
        piped: bool,
    ) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_near"));
        command
            .current_dir(self.home.path())
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path())
            .env("APPDATA", self.home.path())
            .env("NO_COLOR", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if offline {
            command.arg("--offline");
        }
        if quiet {
            command.arg("--quiet");
        }
        command.args(["tokens", "sender.near", transfer]);
        match transfer {
            "send-near" => {
                command.args([receiver, "1 NEAR"]);
            }
            "send-ft" | "send-ft-call" => {
                command.args(["ft.near", receiver, "1 TEST", "memo", ""]);
                if transfer == "send-ft-call" {
                    command.args(["msg-args", "{}"]);
                }
            }
            "send-nft" => {
                command.args(["nft.near", receiver, "token-1"]);
            }
            _ => panic!("unexpected transfer: {transfer}"),
        }
        command.args([
            "network-config",
            "mainnet",
            "sign-later",
            "--signer-public-key",
            "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp",
            "--nonce",
            "0",
            "--block-hash",
            "11111111111111111111111111111111",
            "display",
        ]);
        command.stdin(if piped { Stdio::piped() } else { Stdio::null() });
        let mut child = command.spawn().unwrap();
        if piped {
            // Piped approval text must never count as an interactive confirmation.
            let _ = child.stdin.take().unwrap().write_all(b"yes\n");
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while child.try_wait().unwrap().is_none() {
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                let output = child.wait_with_output().unwrap();
                panic!(
                    "{transfer} hung with non-TTY stdin: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        child.wait_with_output().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        self.server.take().unwrap().join().unwrap();
    }
}

const TRANSFERS: [&str; 4] = ["send-near", "send-ft", "send-ft-call", "send-nft"];

fn assert_rejected(output: &Output, warning: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "unexpected success: {stderr}");
    assert!(stderr.contains(warning), "missing warning: {stderr}");
    assert!(
        stderr.contains("stdin is not a terminal"),
        "missing actionable error: {stderr}"
    );
    assert!(
        stderr.contains("interactive terminal"),
        "missing recovery advice: {stderr}"
    );
    assert!(
        !stderr.contains("Do you want to proceed?"),
        "prompt attempted: {stderr}"
    );
    assert!(
        !stderr.contains("Unsigned transaction:"),
        "transaction constructed: {stderr}"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Transaction hash to sign:"));
}

#[test]
fn non_tty_offline_and_wrong_network_warnings_fail_for_every_transfer() {
    let fixture = Fixture::new(true);
    for transfer in TRANSFERS {
        for piped in [false, true] {
            assert_rejected(
                &fixture.run(transfer, "receiver.near", true, false, piped),
                "Skipping account validation for <receiver.near> on <mainnet> in offline mode.",
            );
            assert_rejected(
                &fixture.run(transfer, "receiver.testnet", true, false, piped),
                "<receiver.testnet> looks like it belongs to a different network than <mainnet>.",
            );
        }
    }
    assert!(
        fixture.requests.lock().unwrap().is_empty(),
        "offline mode made RPC requests"
    );
}

#[test]
fn non_tty_missing_recipients_fail_before_signing_for_every_transfer() {
    let fixture = Fixture::new(false);
    for transfer in TRANSFERS {
        for piped in [false, true] {
            let start = fixture.requests.lock().unwrap().len();
            assert_rejected(
                &fixture.run(transfer, "receiver.near", false, false, piped),
                "<receiver.near> does not exist on <mainnet>.",
            );
            let requests = fixture.requests.lock().unwrap();
            let requests = &requests[start..];
            assert_eq!(
                requests
                    .iter()
                    .filter(|r| r["params"]["account_id"] == "receiver.near")
                    .count(),
                1
            );
            assert!(
                !requests
                    .iter()
                    .any(|r| r["params"]["method_name"] == "storage_balance_of"),
                "transfer construction continued after failed recipient validation"
            );
        }
    }
}

#[test]
fn non_tty_valid_recipients_still_prepare_each_transfer() {
    let fixture = Fixture::new(true);
    for transfer in TRANSFERS {
        let output = fixture.run(transfer, "receiver.near", false, false, false);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{transfer}: {stderr}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let encoded = stdout
            .split("Unsigned transaction (serialized as base64):\n")
            .nth(1)
            .unwrap()
            .lines()
            .next()
            .unwrap();
        let transaction = encoded
            .parse::<near_cli_rs::types::transaction::TransactionAsBase64>()
            .unwrap()
            .inner;
        assert_eq!(transaction.signer_id.as_str(), "sender.near");
        match transaction.actions.as_slice() {
            [near_primitives::transaction::Action::Transfer(action)] if transfer == "send-near" => {
                assert_eq!(transaction.receiver_id.as_str(), "receiver.near");
                assert_eq!(action.deposit, near_token::NearToken::from_near(1));
            }
            [near_primitives::transaction::Action::FunctionCall(action)] => {
                assert_eq!(
                    transaction.receiver_id.as_str(),
                    if transfer == "send-nft" {
                        "nft.near"
                    } else {
                        "ft.near"
                    }
                );
                assert_eq!(
                    action.method_name,
                    match transfer {
                        "send-ft" => "ft_transfer",
                        "send-ft-call" => "ft_transfer_call",
                        "send-nft" => "nft_transfer",
                        _ => panic!("unexpected function call"),
                    }
                );
                let args: Value = serde_json::from_slice(&action.args).unwrap();
                assert_eq!(args["receiver_id"], "receiver.near");
            }
            actions => panic!("unexpected {transfer} actions: {actions:?}"),
        }
    }
    let requests = fixture.requests.lock().unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|r| r["params"]["request_type"] == "view_account"
                && r["params"]["account_id"] == "receiver.near")
            .count(),
        4
    );
}

#[test]
fn explicit_quiet_mode_preserves_existing_validation_opt_out() {
    let fixture = Fixture::new(false);
    for transfer in TRANSFERS {
        let output = fixture.run(transfer, "receiver.testnet", true, true, false);
        assert!(
            output.status.success(),
            "{transfer}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("Unsigned transaction (serialized as base64):")
        );
    }
    assert!(fixture.requests.lock().unwrap().is_empty());
}
