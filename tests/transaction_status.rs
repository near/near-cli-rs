// Windows resolves its config directory with SHGetKnownFolderPath, ignoring APPDATA.
// Run only where the child config can be isolated using environment variables.
#![cfg(unix)]

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    process::Command,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

const HASH: &str = "11111111111111111111111111111111";

fn final_outcome() -> Value {
    json!({
        "final_execution_status": "FINAL",
        "status": {"SuccessValue": ""},
        "transaction": {
            "signer_id": "alice.near", "public_key": format!("ed25519:{HASH}"),
            "nonce": 1, "receiver_id": "bob.near", "actions": [],
            "signature": format!("ed25519:{}", "1".repeat(64)), "hash": HASH
        },
        "transaction_outcome": {
            "proof": [], "block_hash": HASH, "id": HASH,
            "outcome": {
                "logs": ["a log with \"quotes\""], "receipt_ids": [], "gas_burnt": 0,
                "tokens_burnt": "0", "executor_id": "alice.near",
                "status": {"SuccessValue": ""}
            }
        },
        "receipts_outcome": []
    })
}

#[test]
fn transaction_status_prints_json_on_stdout() {
    let mut failure = final_outcome();
    failure["status"] = json!({"Failure": {"ActionError": {
        "index": 0, "kind": {"AccountDoesNotExist": {"account_id": "missing.near"}}
    }}});
    for response in [
        json!({"final_execution_status": "NONE"}),
        final_outcome(),
        failure,
    ] {
        for mode in ["normal", "quiet", "teach-me"] {
            assert_output_contract(response.clone(), mode);
        }
    }
}

fn assert_output_contract(response: Value, mode: &str) {
    let quiet = mode == "quiet";
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let rpc_response = response.clone();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "CLI did not request transaction status"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("{error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        let mut reader = BufReader::new(&stream);
        let mut content_length = None;
        loop {
            let mut line = String::new();
            assert_ne!(reader.read_line(&mut line).unwrap(), 0);
            if line == "\r\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                content_length = Some(value.trim().parse::<usize>().unwrap());
            }
        }
        let mut body = vec![0; content_length.unwrap()];
        reader.read_exact(&mut body).unwrap();
        let request: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(request["method"], "tx");
        assert_eq!(request["params"]["tx_hash"], HASH);
        assert_eq!(request["params"]["wait_until"], "FINAL");
        let body =
            json!({"jsonrpc": "2.0", "id": request["id"], "result": rpc_response}).to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    });

    let temp = tempfile::tempdir().unwrap();
    let config_dir = if cfg!(target_os = "macos") {
        temp.path().join("Library/Application Support/near-cli")
    } else {
        temp.path().join("near-cli")
    };
    std::fs::create_dir_all(&config_dir).unwrap();
    let credentials = temp.path().join("credentials");
    std::fs::create_dir_all(&credentials).unwrap();
    // Avoid the unrelated token-list download at startup.
    std::fs::write(credentials.join("ft_contracts.json"), "[]").unwrap();
    let config = format!(
        "version = \"5\"\ncredentials_home_dir = {}\n[network_connection.mock]\nnetwork_name = \"mock\"\nrpc_url = {url:?}\nwallet_url = {url:?}\nexplorer_transaction_url = {url:?}\n",
        toml::Value::String(credentials.to_string_lossy().into_owned())
    );
    std::fs::write(config_dir.join("config.toml"), config).unwrap();
    // Refuse background self-update requests locally; only the mock RPC bypasses
    // the proxy. Port zero cannot be a listening endpoint.
    let mut command = Command::new(env!("CARGO_BIN_EXE_near"));
    command
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("HTTPS_PROXY", "http://127.0.0.1:0")
        .env("https_proxy", "http://127.0.0.1:0")
        .env("HTTP_PROXY", "http://127.0.0.1:0")
        .env("http_proxy", "http://127.0.0.1:0")
        .env("ALL_PROXY", "http://127.0.0.1:0")
        .env("all_proxy", "http://127.0.0.1:0")
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost");
    if mode != "normal" {
        command.arg(format!("--{mode}"));
    }
    let output = command
        .args(["transaction", "view-status", HASH, "network-config", "mock"])
        .output()
        .unwrap();
    server.join().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stderr}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Teach-me intentionally includes existing RPC tracing on stdout.
    let json_output = if mode == "teach-me" {
        assert!(stdout.contains("JSON Request Body:"));
        let start = stdout.rfind("\n{\n").expect("pretty JSON after tracing") + 1;
        &stdout[start..]
    } else {
        &stdout
    };
    let parsed: Value = serde_json::from_str(json_output).expect("response must be JSON");
    let expected: near_jsonrpc_client::methods::tx::RpcTransactionResponse =
        serde_json::from_value(response.clone()).unwrap();
    assert_eq!(parsed, serde_json::to_value(expected).unwrap());
    assert_eq!(
        parsed["final_execution_status"],
        response["final_execution_status"]
    );
    assert_eq!(parsed.get("status"), response.get("status"));
    if response.get("transaction").is_some() {
        assert_eq!(parsed["transaction"]["hash"], HASH);
        assert_eq!(
            parsed["transaction_outcome"]["outcome"]["logs"],
            response["transaction_outcome"]["outcome"]["logs"]
        );
    }
    assert_eq!(stderr.contains("Transaction status:"), !quiet);
    if quiet {
        assert_eq!(stdout.lines().count(), 1);
    } else {
        assert!(stdout.lines().count() > 1);
    }
}
