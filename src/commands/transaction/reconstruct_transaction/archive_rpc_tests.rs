use super::*;
use crate::commands::contract::download_wasm::ContractType;
use near_primitives::{hash::CryptoHash, types::BlockReference};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};

/// Serve one block with one shard, then code at heights 100..=102.
fn with_archive_rpc<T>(
    codes: Vec<Vec<u8>>,
    run: impl FnOnce(&crate::config::NetworkConfig) -> T,
) -> (T, Vec<Value>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let mut requests = vec![];
        while matches!(
            stopped.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ) {
            let (mut stream, _) = match listener.accept() {
                Ok(connection) => connection,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }
                Err(err) => panic!("accept: {err}"),
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    content_length = value.trim().parse::<usize>().unwrap();
                }
            }
            let mut body = vec![0; content_length];
            reader.read_exact(&mut body).unwrap();
            let request: Value = serde_json::from_slice(&body).unwrap();
            let result = if request["method"] == "block" {
                let hash = CryptoHash::default();
                json!({
                    "author": "validator.near",
                    "header": near_primitives::views::BlockHeaderView { height: 100, ..Default::default() },
                    "chunks": [{
                        "chunk_hash": hash, "prev_block_hash": hash, "outcome_root": hash,
                        "prev_state_root": hash, "encoded_merkle_root": hash,
                        "encoded_length": 0, "height_created": 100, "height_included": 100,
                        "shard_id": 0, "gas_used": 0, "gas_limit": 0,
                        "balance_burnt": "0", "outgoing_receipts_root": hash, "tx_root": hash,
                        "validator_proposals": [],
                        "signature": near_crypto::Signature::empty(near_crypto::KeyType::ED25519)
                    }]
                })
            } else {
                assert_eq!(request["method"], "query");
                let height = request["params"]["block_id"].as_u64().unwrap();
                let code = codes[(height - 100) as usize].clone();
                let mut result = serde_json::to_value(near_primitives::views::ContractCodeView {
                    hash: CryptoHash::hash_bytes(&code),
                    code,
                })
                .unwrap();
                result["block_height"] = json!(height);
                result["block_hash"] = json!(CryptoHash::default());
                result
            };
            let response =
                json!({"jsonrpc": "2.0", "id": request["id"], "result": result}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
            requests.push(request);
        }
        requests
    });
    let mut config = crate::config::Config::default()
        .network_connection
        .values()
        .next()
        .unwrap()
        .clone();
    config.rpc_url = format!("http://{address}").parse().unwrap();
    config.rpc_api_key = None;
    let result = run(&config);
    stop.send(()).unwrap();
    (result, server.join().unwrap())
}

#[test]
fn reconstruction_scans_past_empty_and_old_code() {
    let code = b"\0asm\x01\0\0\0".to_vec();
    let block_hash = CryptoHash::hash_bytes(b"transaction outcome block");
    let dir = tempfile::tempdir().unwrap();
    let path = crate::types::path_buf::PathBuf(dir.path().join("contract.wasm"));
    let (result, requests) =
        with_archive_rpc(vec![vec![], b"old code".to_vec(), code.clone()], |config| {
            download_code(
                &ContractType::Regular("receiver.near".parse().unwrap()),
                config,
                BlockReference::BlockId(near_primitives::types::BlockId::Hash(block_hash)),
                &path,
                &CryptoHash::hash_bytes(&code),
            )
        });
    result.unwrap();
    assert_eq!(requests[0]["method"], "block");
    assert_eq!(requests[0]["params"]["block_id"], json!(block_hash));
    assert_eq!(std::fs::read(path.0).unwrap(), code);
    assert_eq!(
        requests[1..]
            .iter()
            .map(|r| r["params"]["block_id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [100, 101, 102]
    );
}

#[test]
fn reconstruction_exhausts_mismatched_code_without_overwriting_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = crate::types::path_buf::PathBuf(dir.path().join("contract.wasm"));
    std::fs::write(&path.0, b"existing file").unwrap();
    let (result, requests) = with_archive_rpc(
        vec![vec![], b"old code".to_vec(), b"other code".to_vec()],
        |config| {
            download_code(
                &ContractType::Regular("receiver.near".parse().unwrap()),
                config,
                BlockReference::latest(),
                &path,
                &CryptoHash::hash_bytes(b"expected code"),
            )
        },
    );
    assert!(format!("{:#}", result.unwrap_err()).contains("after trying 3 block heights"));
    assert_eq!(std::fs::read(path.0).unwrap(), b"existing file");
    assert_eq!(
        requests.len(),
        4,
        "one block and all three code heights must be requested"
    );
}

#[test]
fn ordinary_download_still_returns_the_first_code_response() {
    let (result, requests) = with_archive_rpc(vec![vec![]], |config| {
        crate::commands::contract::download_wasm::get_code(
            &ContractType::Regular("receiver.near".parse().unwrap()),
            config,
            BlockReference::latest(),
        )
    });
    assert!(result.unwrap().is_empty());
    assert_eq!(requests.len(), 2);
}

#[test]
fn global_downloads_keep_their_hash_filter() {
    let code = b"\0asm\x01\0\0\0".to_vec();
    let code_hash = CryptoHash::hash_bytes(&code);
    for contract_type in [
        ContractType::GlobalContractByCodeHash(code_hash),
        ContractType::GlobalContractByAccountId {
            account_id: "receiver.near".parse().unwrap(),
            code_hash: Some(code_hash),
        },
    ] {
        let (result, requests) = with_archive_rpc(vec![vec![], code.clone()], |config| {
            crate::commands::contract::download_wasm::get_code(
                &contract_type,
                config,
                BlockReference::latest(),
            )
        });
        assert_eq!(result.unwrap(), code);
        assert_eq!(requests.len(), 3);
    }
}

#[test]
fn conflicting_global_hashes_fail_before_rpc() {
    let (result, requests) = with_archive_rpc(vec![], |config| {
        crate::commands::contract::download_wasm::get_code_with_hash(
            &ContractType::GlobalContractByCodeHash(CryptoHash::hash_bytes(b"global code")),
            config,
            BlockReference::latest(),
            Some(CryptoHash::hash_bytes(b"other code")),
        )
    });
    assert!(result.unwrap_err().to_string().contains("conflicts"));
    assert!(requests.is_empty());
}
