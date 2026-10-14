use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use near_cli_rs::config::Config;
use near_cli_rs::types::transaction::TransactionAsBase64;
use near_crypto::{KeyType, PublicKey, PublicKeyHandle, SecretKey};
use near_primitives::transaction::Action;
use serde_json::{Value, json};

struct RpcFixture {
    address: std::net::SocketAddr,
    requests: Arc<Mutex<Vec<Value>>>,
    stopping: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl RpcFixture {
    fn new(pages: Vec<Vec<PublicKey>>, error: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopping = Arc::new(AtomicBool::new(false));
        let worker = std::thread::spawn({
            let requests = requests.clone();
            let stopping = stopping.clone();
            move || {
                let mut page_index = 0;
                for connection in listener.incoming() {
                    let mut connection = connection.unwrap();
                    if stopping.load(Ordering::SeqCst) {
                        break;
                    }
                    connection
                        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                        .unwrap();
                    let mut reader = BufReader::new(connection.try_clone().unwrap());
                    let mut content_length = 0;
                    loop {
                        let mut line = String::new();
                        reader.read_line(&mut line).unwrap();
                        if line == "\r\n" {
                            break;
                        }
                        if let Some(length) =
                            line.to_ascii_lowercase().strip_prefix("content-length:")
                        {
                            content_length = length.trim().parse().unwrap();
                        }
                    }
                    let mut body = vec![0; content_length];
                    reader.read_exact(&mut body).unwrap();
                    let request: Value = serde_json::from_slice(&body).unwrap();
                    requests.lock().unwrap().push(request.clone());
                    let response = if error {
                        json!({"jsonrpc": "2.0", "id": request["id"], "error": {
                            "code": -32000, "message": "Server error", "data": {
                                "name": "HANDLER_ERROR", "cause": {
                                    "name": "UNKNOWN_ACCOUNT", "info": {"account_id": "demo.near"}
                                }
                            }
                        }})
                    } else {
                        let keys = &pages[page_index];
                        let last_key = if page_index + 1 < pages.len() {
                            Some(PublicKeyHandle::from(keys.last().unwrap()).to_string())
                        } else {
                            None
                        };
                        page_index += 1;
                        json!({"jsonrpc": "2.0", "id": request["id"], "result": {
                            "keys": keys.iter().map(|key| json!({
                                "public_key": PublicKeyHandle::from(key).to_string(),
                                "access_key": {"nonce": 0, "permission": "FullAccess"}
                            })).collect::<Vec<_>>(),
                            "last_key": last_key,
                            "block_height": 10,
                            "block_hash": "11111111111111111111111111111111"
                        }})
                    };
                    let body = response.to_string();
                    write!(connection, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
                }
            }
        });
        Self {
            address,
            requests,
            stopping,
            worker: Some(worker),
        }
    }
}

impl Drop for RpcFixture {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.address);
        self.worker.take().unwrap().join().unwrap();
    }
}

fn key(seed: &str) -> PublicKey {
    SecretKey::from_seed(KeyType::ED25519, seed).public_key()
}

fn delete_keys(
    rpc: &RpcFixture,
    keys: &[PublicKey],
    offline: bool,
) -> (Output, Option<Vec<Action>>) {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = if cfg!(target_os = "macos") {
        temp.path().join("Library/Application Support/near-cli")
    } else {
        temp.path().join("near-cli")
    };
    std::fs::create_dir_all(&config_dir).unwrap();
    let mut config = Config {
        credentials_home_dir: temp.path().join("credentials"),
        ..Config::default()
    };
    let mut network = config.network_connection.remove("mainnet").unwrap();
    network.rpc_url = format!("http://{}/", rpc.address).parse().unwrap();
    config.network_connection.clear();
    // Config aliases need not be the chain's network_name.
    config
        .network_connection
        .insert("custom-mainnet".to_string(), network);
    std::fs::create_dir_all(&config.credentials_home_dir).unwrap();
    std::fs::write(config.credentials_home_dir.join("ft_contracts.json"), "[]").unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        toml::to_string(&config.into_latest_version()).unwrap(),
    )
    .unwrap();

    let transaction_file = temp.path().join("unsigned.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_near"));
    command
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("APPDATA", temp.path())
        .stdin(Stdio::null());
    if offline {
        command.arg("--offline");
    }
    let keys = keys
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let output = command
        .args([
            "account",
            "delete-keys",
            "demo.near",
            "public-keys",
            &keys,
            "network-config",
            "custom-mainnet",
            "sign-later",
            "--signer-public-key",
            &key("signer").to_string(),
            "--nonce",
            "1",
            "--block-hash",
            "11111111111111111111111111111111",
            "save-to-file",
        ])
        .arg(&transaction_file)
        .output()
        .unwrap();
    let actions = transaction_file.exists().then(|| {
        let saved: Value =
            serde_json::from_slice(&std::fs::read(transaction_file).unwrap()).unwrap();
        let transaction: TransactionAsBase64 = saved["Unsigned transaction (serialized as base64)"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let transaction: near_primitives::transaction::Transaction = transaction.into();
        assert_eq!(transaction.signer_id().as_str(), "demo.near");
        assert_eq!(transaction.receiver_id().as_str(), "demo.near");
        transaction.take_actions()
    });
    (output, actions)
}

fn assert_rejected(result: (Output, Option<Vec<Action>>), message: &str) -> String {
    let (output, actions) = result;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains(message), "{stderr}");
    assert!(!stderr.contains("Unsigned transaction:"), "{stderr}");
    assert!(actions.is_none());
    stderr.into_owned()
}

#[test]
fn rejects_mixed_network_and_wrong_network_keys_before_transaction_preparation() {
    let mainnet_key = key("mainnet");
    let testnet_key = key("testnet");
    for requested in [
        vec![mainnet_key.clone(), testnet_key.clone()],
        vec![testnet_key.clone()],
    ] {
        let rpc = RpcFixture::new(vec![vec![mainnet_key.clone()]], false);
        let stderr = assert_rejected(
            delete_keys(&rpc, &requested, false),
            "Cannot delete access keys for <demo.near> on network <mainnet>",
        );
        assert!(stderr.contains(&testnet_key.to_string()), "{stderr}");
        let requests = rpc.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]["params"]["request_type"],
            "view_access_key_list"
        );
        assert_eq!(requests[0]["params"]["account_id"], "demo.near");
    }
}

#[test]
fn accepts_all_keys_on_selected_network_including_paginated_and_post_quantum_keys() {
    let requested = vec![
        key("shared-on-both-networks"),
        key("mainnet"),
        SecretKey::from_random(KeyType::MLDSA65).public_key(),
    ];
    let rpc = RpcFixture::new(
        vec![requested[..1].to_vec(), requested[1..].to_vec()],
        false,
    );
    let (output, actions) = delete_keys(&rpc, &requested, false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = actions
        .unwrap()
        .into_iter()
        .map(|action| match action {
            Action::DeleteKey(action) => action.public_key,
            _ => panic!("expected DeleteKey action"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, requested);
    let requests = rpc.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["params"]["finality"], "final");
    assert_eq!(
        requests[1]["params"]["block_id"],
        "11111111111111111111111111111111"
    );
    assert_eq!(
        requests[1]["params"]["after_key"],
        PublicKeyHandle::from(&requested[0]).to_string()
    );
}

#[test]
fn fails_closed_when_selected_network_cannot_be_queried() {
    let rpc = RpcFixture::new(vec![], true);
    assert_rejected(
        delete_keys(&rpc, &[key("mainnet")], false),
        "Failed to fetch access keys for <demo.near> on network <mainnet>",
    );
}

#[test]
fn rejects_keys_when_account_has_no_access_keys() {
    let rpc = RpcFixture::new(vec![vec![]], false);
    assert_rejected(
        delete_keys(&rpc, &[key("mainnet")], false),
        "does not exist on this network",
    );
}

#[test]
fn offline_mode_keeps_manual_transaction_construction_without_rpc() {
    let requested = vec![key("mainnet"), key("testnet")];
    let rpc = RpcFixture::new(vec![], true);
    let (output, actions) = delete_keys(&rpc, &requested, true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = actions
        .unwrap()
        .into_iter()
        .map(|action| match action {
            Action::DeleteKey(action) => action.public_key,
            _ => panic!("expected DeleteKey action"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, requested);
    assert!(rpc.requests.lock().unwrap().is_empty());
}
