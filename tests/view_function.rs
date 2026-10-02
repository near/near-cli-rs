use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use near_cli_rs::config::Config;
use serde_json::{Value, json};

const BLOCK_HEIGHT: u64 = 123456;
const BLOCK_HASH: &str = "11111111111111111111111111111112";

fn call_result(bytes: &[u8]) -> Value {
    json!({"result": {
        "block_height": BLOCK_HEIGHT,
        "block_hash": BLOCK_HASH,
        "logs": ["view log"],
        "result": bytes,
    }})
}

fn run_near(args: &[&str], responses: Vec<Value>) -> (Output, Vec<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let rpc_url = format!("http://{}/", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for mut response in responses {
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "missing RPC request");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept failed: {err}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(15)))
                .unwrap();
            let mut reader = BufReader::new(&stream);
            let mut content_length = None;
            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
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
            response["id"] = request["id"].clone();
            response["jsonrpc"] = json!("2.0");
            let body = serde_json::to_vec(&response).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
            requests.push(request);
        }
        requests
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = temp_dir.path();
    let config_dir = if cfg!(target_os = "macos") {
        config_home.join("Library/Application Support/near-cli")
    } else {
        config_home.join("near-cli")
    };
    std::fs::create_dir_all(&config_dir).unwrap();
    let mut config = Config {
        credentials_home_dir: config_home.join("credentials"),
        ..Config::default()
    };
    let mut network = config.network_connection.remove("testnet").unwrap();
    network.rpc_url = rpc_url.parse().unwrap();
    config.network_connection.clear();
    config.network_connection.insert("testnet".into(), network);
    std::fs::create_dir_all(&config.credentials_home_dir).unwrap();
    std::fs::write(config.credentials_home_dir.join("ft_contracts.json"), "[]").unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        toml::to_string(&config.into_latest_version()).unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_near"))
        .current_dir(config_home)
        .env("HOME", config_home)
        .env("XDG_CONFIG_HOME", config_home)
        .env("APPDATA", config_home)
        // Keep the background update check offline while allowing the mock RPC.
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "127.0.0.1")
        .args(args)
        .output()
        .unwrap();
    (output, server.join().unwrap())
}

fn view_args<'a>(mode: &'a [&'a str], block: &'a [&'a str]) -> Vec<&'a str> {
    let mut args = mode.to_vec();
    args.extend([
        "contract",
        "call-function",
        "as-read-only",
        "contract.testnet",
        "get_value",
        "json-args",
        "{}",
        "network-config",
        "testnet",
    ]);
    args.extend(block);
    args
}

#[test]
fn view_output_preserves_return_values_and_reports_execution_block() {
    let results: &[(&[u8], &str)] = &[
        (br#"{"answer":42}"#, "{\n  \"answer\": 42\n}\n"),
        (b"hello\nworld", "hello\nworld\n"),
        (b"", "Empty return value\n"),
        (
            &[0, 255, 128],
            "The returned value is not printable (binary data)\n",
        ),
    ];
    let blocks: &[(&[&str], Value)] = &[
        (&["now"], json!({"finality": "final"})),
        (&["at-block-height", "123460"], json!({"block_id": 123460})),
        (
            &["at-block-hash", BLOCK_HASH],
            json!({"block_id": BLOCK_HASH}),
        ),
    ];
    for mode in [&[][..], &["--quiet"][..], &["--teach-me"][..]] {
        for (block, selection) in blocks {
            for (raw, formatted) in results {
                let (output, requests) = run_near(&view_args(mode, block), vec![call_result(raw)]);
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(output.status.success(), "{mode:?} {block:?}: {stderr}");
                assert_eq!(requests.len(), 1);
                assert_eq!(requests[0]["method"], "query");
                assert_eq!(requests[0]["params"]["request_type"], "call_function");
                assert_eq!(requests[0]["params"]["account_id"], "contract.testnet");
                assert_eq!(requests[0]["params"]["method_name"], "get_value");
                assert_eq!(requests[0]["params"]["args_base64"], "e30=");
                for (key, value) in selection.as_object().unwrap() {
                    assert_eq!(&requests[0]["params"][key], value);
                }
                if mode == ["--quiet"] {
                    assert_eq!(&output.stdout, raw);
                    assert!(stderr.is_empty(), "{stderr}");
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(!stdout.contains("Block height:"), "{stdout}");
                    assert!(!stdout.contains("Block hash:"), "{stdout}");
                    assert_eq!(
                        stderr.matches("Block height: 123456").count(),
                        1,
                        "{stderr}"
                    );
                    assert_eq!(
                        stderr.matches(&format!("Block hash: {BLOCK_HASH}")).count(),
                        1,
                        "{stderr}"
                    );
                    assert!(
                        stderr.contains("Function execution return value (printed to stdout):")
                    );
                    if mode == ["--teach-me"] {
                        assert!(stdout.ends_with(formatted), "{stdout}");
                        assert!(stdout.contains("JSON RPC Response:"), "{stdout}");
                        assert!(stdout.contains("\"block_height\": 123456"), "{stdout}");
                        assert!(stdout.contains(BLOCK_HASH), "{stdout}");
                        assert!(stdout.contains("view log"), "{stdout}");
                    } else {
                        assert_eq!(output.stdout, formatted.as_bytes());
                        assert!(stderr.contains("view log"), "{stderr}");
                    }
                }
            }
        }
    }
}

#[test]
fn legacy_view_reports_execution_block() {
    let (output, requests) = run_near(
        &[
            "view",
            "contract.testnet",
            "get_value",
            "{}",
            "--network-id",
            "testnet",
        ],
        vec![call_result(b"42")],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(output.stdout, b"42\n");
    assert!(stderr.contains("Block height: 123456"), "{stderr}");
    assert!(
        stderr.contains(&format!("Block hash: {BLOCK_HASH}")),
        "{stderr}"
    );
    assert_eq!(requests.len(), 1);
}

#[test]
fn failed_view_does_not_report_success_metadata() {
    for (response, expected) in [
        (
            json!({"error": {"code": -32000, "message": "Server error", "data": "view execution failed"}}),
            "Read-only function execution failed",
        ),
        (
            json!({"result": {"block_height": BLOCK_HEIGHT, "block_hash": BLOCK_HASH, "keys": []}}),
            "Received unexpected query kind",
        ),
    ] {
        for mode in [&[][..], &["--quiet"][..], &["--teach-me"][..]] {
            let (output, _) = run_near(&view_args(mode, &["now"]), vec![response.clone()]);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!output.status.success(), "{stderr}");
            assert!(!stdout.contains("Block height:"), "{stdout}");
            assert!(!stdout.contains("Block hash:"), "{stdout}");
            if mode != ["--teach-me"] {
                assert!(output.stdout.is_empty());
            }
            assert!(
                stderr.contains("Failed to fetch query for read-only function call: 'get_value'"),
                "{stderr}"
            );
            assert!(stderr.contains(expected), "{stderr}");
            assert!(!stderr.contains("Block height:"), "{stderr}");
            assert!(!stderr.contains("Block hash:"), "{stderr}");
            assert!(
                !stderr.contains("Function execution return value"),
                "{stderr}"
            );
        }
    }
}

#[test]
fn incidental_token_view_does_not_print_execution_metadata() {
    let (output, requests) = run_near(
        &[
            "tokens",
            "owner.testnet",
            "view-ft-balance",
            "token.testnet",
            "network-config",
            "testnet",
            "now",
        ],
        vec![
            call_result(br#"{"spec":"ft-1.0.0","name":"Test Token","symbol":"TEST","decimals":0}"#),
            call_result(br#""42""#),
        ],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "<owner.testnet> account has 42 TEST  (FT-contract: token.testnet)\n"
    );
    assert!(!stderr.contains("Block height:"), "{stderr}");
    assert!(!stderr.contains("Block hash:"), "{stderr}");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["params"]["method_name"], "ft_metadata");
    assert_eq!(requests[1]["params"]["method_name"], "ft_balance_of");
}
