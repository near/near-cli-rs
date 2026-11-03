use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use near_cli_rs::config::Config;
use near_primitives::types::StoreKey;
use near_primitives::views::{StateItem, ViewStateResult};
use serde_json::{Value, json};

const BLOCK_HASH: &str = "11111111111111111111111111111111";

/// Serves `view_state` pages in order, linking each page to the next with `last_key`.
struct RpcFixture {
    address: std::net::SocketAddr,
    requests: Arc<Mutex<Vec<Value>>>,
    stopping: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl RpcFixture {
    fn new(pages: Vec<Vec<(&'static str, &'static str)>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopping = Arc::new(AtomicBool::new(false));
        let worker = std::thread::spawn({
            let requests = requests.clone();
            let stopping = stopping.clone();
            move || {
                for (page_index, connection) in listener.incoming().enumerate() {
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
                    let items = &pages[page_index];
                    let last_key = (page_index + 1 < pages.len())
                        .then(|| StoreKey::from(items.last().unwrap().0.as_bytes().to_vec()));
                    let page = ViewStateResult {
                        values: items
                            .iter()
                            .map(|(key, value)| StateItem {
                                key: StoreKey::from(key.as_bytes().to_vec()),
                                value: value.as_bytes().to_vec().into(),
                            })
                            .collect(),
                        proof: Vec::new(),
                        last_key,
                    };
                    let mut result = serde_json::to_value(page).unwrap();
                    result["block_height"] = json!(10);
                    result["block_hash"] = json!(BLOCK_HASH);
                    let response = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
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

fn view_storage(rpc: &RpcFixture, output_format: &str) -> Output {
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

    let output = Command::new(env!("CARGO_BIN_EXE_near"))
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("APPDATA", temp.path())
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .args([
            "contract",
            "view-storage",
            "demo.near",
            "keys-start-with-string",
            "k",
            output_format,
            "network-config",
            "custom-mainnet",
            "now",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn json_values(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let values = stdout
        .split_once("Contract state (values):\n")
        .and_then(|(_, rest)| rest.split_once("\n\nContract state (proof):"))
        .unwrap_or_else(|| panic!("{stdout}"))
        .0;
    serde_json::from_str(values).unwrap()
}

fn base64(value: &str) -> String {
    serde_json::to_value(StoreKey::from(value.as_bytes().to_vec()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn follows_last_key_and_pins_later_pages_to_the_first_block() {
    let rpc = RpcFixture::new(vec![
        vec![("k1", "v1"), ("k2", "v2")],
        vec![("k3", "v3")],
        vec![("k4", "v4")],
    ]);
    let output = view_storage(&rpc, "as-json");
    let expected = ["k1", "k2", "k3", "k4"]
        .iter()
        .zip(["v1", "v2", "v3", "v4"])
        .map(|(key, value)| json!({"key": base64(key), "value": base64(value)}))
        .collect::<Vec<_>>();
    assert_eq!(json_values(&output), json!(expected));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Contract state (proof):\n[]\n"));

    let requests = rpc.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    for (index, request) in requests.iter().enumerate() {
        let params = &request["params"];
        assert_eq!(params["request_type"], "view_state");
        assert_eq!(params["account_id"], "demo.near");
        assert_eq!(params["prefix_base64"], base64("k"));
        assert!(params["limit"].as_u64().unwrap() > 0, "{params}");
        if index == 0 {
            assert_eq!(params["finality"], "final");
            assert!(params.get("after_key_base64").is_none(), "{params}");
        } else {
            assert_eq!(params["block_id"], BLOCK_HASH);
            assert!(params.get("finality").is_none(), "{params}");
        }
    }
    assert_eq!(requests[1]["params"]["after_key_base64"], base64("k2"));
    assert_eq!(requests[2]["params"]["after_key_base64"], base64("k3"));
}

#[test]
fn stops_after_a_page_without_last_key() {
    let rpc = RpcFixture::new(vec![vec![("k1", "v1")]]);
    let output = view_storage(&rpc, "as-json");
    assert_eq!(
        json_values(&output),
        json!([{"key": base64("k1"), "value": base64("v1")}])
    );
    assert_eq!(rpc.requests.lock().unwrap().len(), 1);
}

#[test]
fn text_output_reports_empty_state() {
    let rpc = RpcFixture::new(vec![vec![]]);
    let output = view_storage(&rpc, "as-text");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Contract state (values):\n\tempty state\n\t---"),
        "{stdout}"
    );
    assert!(stdout.contains("Contract state (proof):\n[]\n"), "{stdout}");
    assert_eq!(rpc.requests.lock().unwrap().len(), 1);
}
