use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use clap::Parser;
use interactive_clap::{FromCli, ResultFromCli, ToCli};
use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi};
use near_cli_rs::commands::TopLevelCommand;
use near_cli_rs::{GlobalContext, Verbosity};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: <TopLevelCommand as ToCli>::CliVariant,
}

// This integration-test process never creates a native keychain entry. Unlike
// keyring's entry-local mock, this store lets the real signer reopen saved keys.
#[derive(Default)]
struct Store {
    entries: Mutex<BTreeMap<(String, String), Arc<keyring::mock::MockCredential>>>,
    failure: Mutex<Option<&'static str>>,
}

struct Builder(Arc<Store>);
struct StoredCredential(Arc<keyring::mock::MockCredential>);

impl CredentialApi for StoredCredential {
    fn set_secret(&self, secret: &[u8]) -> keyring::Result<()> {
        self.0.set_secret(secret)
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.0.get_secret()
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        self.0.delete_credential()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CredentialBuilderApi for Builder {
    fn build(
        &self,
        _target: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<Credential>> {
        let failure = *self.0.failure.lock().unwrap();
        if failure == Some("open") {
            return Err(keyring::Error::NoEntry);
        }
        let credential = self
            .0
            .entries
            .lock()
            .unwrap()
            .entry((service.to_owned(), user.to_owned()))
            .or_default()
            .clone();
        if failure == Some("write") {
            credential.set_error(keyring::Error::NoEntry);
        }
        Ok(Box::new(StoredCredential(credential)))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn run(args: &[&str], context: GlobalContext) -> Result<(), color_eyre::Report> {
    let cli = Cli::try_parse_from(std::iter::once("near").chain(args.iter().copied()))
        .expect("command must retain its CLI syntax");
    match TopLevelCommand::from_cli(Some(cli.command), context) {
        ResultFromCli::Ok(_) => Ok(()),
        ResultFromCli::Err(_, err) => Err(err),
        _ => panic!("fully specified command must not prompt"),
    }
}

fn context(credentials_home_dir: &std::path::Path) -> GlobalContext {
    let mut config = near_cli_rs::config::Config {
        credentials_home_dir: credentials_home_dir.to_owned(),
        ..Default::default()
    };
    for (_, network) in config.network_connection.iter_mut() {
        // Account creation and offline signing must not need a funded account
        // or any RPC server.
        network.rpc_url = "http://127.0.0.1:1/".parse().unwrap();
    }
    GlobalContext {
        config,
        offline: true,
        verbosity: Verbosity::Quiet,
    }
}

fn with_rpc_response<T>(
    config: &near_cli_rs::config::NetworkConfig,
    mut response: serde_json::Value,
    run: impl FnOnce(&near_cli_rs::config::NetworkConfig) -> T,
) -> T {
    use std::io::{BufRead, BufReader, Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut config = config.clone();
    config.rpc_url = format!("http://{}", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "RPC request timed out"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(err) => panic!("accept: {err}"),
            }
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
        let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(request["params"]["request_type"], "view_access_key_list");
        response["id"] = request["id"].clone();
        response["jsonrpc"] = serde_json::json!("2.0");
        let body = response.to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    let result = run(&config);
    server.join().unwrap();
    result
}

fn unknown_account(account_id: &near_primitives::types::AccountId) -> serde_json::Value {
    serde_json::json!({"error": {
        "code": -32000, "message": "Server error", "data": "account does not exist",
        "name": "HANDLER_ERROR", "cause": {"name": "UNKNOWN_ACCOUNT", "info": {
            "requested_account_id": account_id, "block_height": 1,
            "block_hash": "11111111111111111111111111111111"
        }}
    }})
}

#[test]
fn generated_account_is_pickable_and_signs_from_keychain_before_funding() {
    let store = Arc::new(Store::default());
    keyring::set_default_credential_builder(Box::new(Builder(store.clone())));
    let temp = tempfile::tempdir().unwrap();
    let credentials_dir = temp.path().join("credentials");
    let global_context = context(&credentials_dir);

    for network in ["testnet", "mainnet"] {
        run(
            &[
                "account",
                "create-account",
                "fund-later",
                "use-auto-generation",
                "save-to-keychain",
                "network-config",
                network,
            ],
            global_context.clone(),
        )
        .unwrap();
        let account = near_cli_rs::common::get_used_account_list(&credentials_dir)
            .front()
            .unwrap()
            .clone();
        assert!(account.used_as_signer);
        let public_key =
            near_crypto::PublicKey::from_near_implicit_account(&account.account_id).unwrap();
        let entry = keyring::Entry::new(
            &format!("near-{network}-{}", account.account_id),
            &format!("{}:{public_key}", account.account_id),
        )
        .unwrap();
        let password = entry.get_password().unwrap();
        let key_pair: near_cli_rs::common::KeyPairProperties =
            serde_json::from_str(&password).unwrap();
        assert_eq!(key_pair.implicit_account_id, account.account_id);
        let signing_key: near_cli_rs::transaction_signature_options::AccountKeyPair =
            serde_json::from_str(&password).unwrap();
        assert!(signing_key.public_key == public_key);
        assert!(signing_key.private_key.public_key() == public_key);
        assert_eq!(
            std::fs::read_dir(&credentials_dir)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect::<Vec<_>>(),
            ["accounts.json"],
        );
        let wrong_network_entry = keyring::Entry::new(
            &format!("near-other-{}", account.account_id),
            &format!("{}:{public_key}", account.account_id),
        )
        .unwrap();
        assert!(matches!(
            wrong_network_entry.get_password(),
            Err(keyring::Error::NoEntry)
        ));

        let network_config = global_context
            .config
            .network_connection
            .get(network)
            .unwrap();
        let payload = near_cli_rs::commands::message::sign_nep413::NEP413Payload {
            message: "unfunded account test".to_owned(),
            nonce: [7; 32],
            recipient: "test-only".to_owned(),
            callback_url: None,
        };
        let signed_message = Arc::new(Mutex::new(None));
        let captured = signed_message.clone();
        let signing_context = near_cli_rs::commands::message::sign_nep413::FinalSignNep413Context {
            global_context: global_context.clone(),
            payload: payload.clone(),
            signer_id: account.account_id.clone(),
            on_after_signing_callback: Arc::new(move |message| {
                *captured.lock().unwrap() = Some(message);
                Ok(())
            }),
        };
        let sign_cli = <near_cli_rs::commands::message::sign_nep413::signature_options::sign_with_keychain::SignKeychain as ToCli>::CliVariant::try_parse_from([
            "sign-with-keychain", "network-config", network,
        ]).unwrap();
        with_rpc_response(
            network_config,
            unknown_account(&account.account_id),
            |config| {
                let mut signing_context = signing_context;
                signing_context
                    .global_context
                    .config
                    .network_connection
                    .insert(network.to_owned(), config.clone());
                assert!(matches!(
                near_cli_rs::commands::message::sign_nep413::signature_options::sign_with_keychain::SignKeychain::from_cli(Some(sign_cli), signing_context),
                ResultFromCli::Ok(_)
            ));
            },
        );
        let signed_message = signed_message.lock().unwrap();
        let signed_message = signed_message.as_ref().unwrap();
        assert_eq!(signed_message.account_id, account.account_id.as_str());
        // Compare against independently signing the same payload with the key
        // read from the isolated store; do not print signing material.
        assert!(
            signed_message.signature
                == near_cli_rs::commands::message::sign_nep413::sign_nep413_payload(
                    &payload,
                    &signing_key.private_key
                )
                .unwrap()
                .to_string()
        );

        let revoked = serde_json::json!({"result": {
            "keys": [], "block_height": 1, "block_hash": "11111111111111111111111111111111"
        }});
        with_rpc_response(network_config, revoked, |config| {
            assert!(
                near_cli_rs::commands::account::export_account::get_account_key_pair_from_keychain(
                    config,
                    &account.account_id
                )
                .is_err()
            );
        });
        let other_rpc_error = serde_json::json!({"error": {
            "code": -32000, "message": "Server error", "data": "block not found",
            "name": "HANDLER_ERROR", "cause": {"name": "UNKNOWN_BLOCK", "info": {
                "block_reference": {"finality": "final"}
            }}
        }});
        with_rpc_response(network_config, other_rpc_error, |config| {
            assert!(
                near_cli_rs::commands::account::export_account::get_account_key_pair_from_keychain(
                    config,
                    &account.account_id
                )
                .is_err()
            );
        });
        let named_id = "named.testnet".parse().unwrap();
        with_rpc_response(network_config, unknown_account(&named_id), |config| {
            assert!(
                near_cli_rs::commands::account::export_account::get_account_key_pair_from_keychain(
                    config, &named_id
                )
                .is_err()
            );
        });
        assert!(
            near_cli_rs::commands::account::export_account::get_account_key_pair_from_keychain(
                network_config,
                &account.account_id
            )
            .is_err()
        );

        let transaction_path = temp.path().join("signed.json");
        run(
            &[
                "transaction",
                "construct-transaction",
                account.account_id.as_str(),
                "receiver-id",
                "receiver.testnet",
                "add-action",
                "transfer",
                "1 NEAR",
                "skip",
                "network-config",
                network,
                "sign-with-keychain",
                "--signer-public-key",
                &public_key.to_string(),
                "--nonce",
                "1",
                "--block-hash",
                "11111111111111111111111111111111",
                "--block-height",
                "1",
                "save-to-file",
                transaction_path.to_str().unwrap(),
            ],
            global_context.clone(),
        )
        .unwrap();
        let signed: near_cli_rs::commands::transaction::send_signed_transaction::FileSignedTransaction =
            serde_json::from_slice(&std::fs::read(&transaction_path).unwrap()).unwrap();
        assert!(
            signed.signed_transaction.signature.verify(
                signed
                    .signed_transaction
                    .transaction
                    .get_hash_and_size()
                    .0
                    .as_ref(),
                &public_key,
            )
        );
        std::fs::remove_file(transaction_path).unwrap();
    }

    for failure in ["open", "write"] {
        *store.failure.lock().unwrap() = Some(failure);
        let failure_dir = temp.path().join(failure);
        let err = run(
            &[
                "account",
                "create-account",
                "fund-later",
                "use-auto-generation",
                "save-to-keychain",
                "network-config",
                "testnet",
            ],
            context(&failure_dir),
        )
        .unwrap_err();
        assert!(err.to_string().contains("No plaintext file was written"));
        assert!(err.to_string().contains("save-to-folder"));
        assert_eq!(std::fs::read_dir(&failure_dir).unwrap().count(), 0);
        assert!(near_cli_rs::common::get_used_account_list(&failure_dir).is_empty());
    }
    *store.failure.lock().unwrap() = None;

    let invalid_dir = temp.path().join("not-a-directory");
    std::fs::write(&invalid_dir, "").unwrap();
    let entry_count = store.entries.lock().unwrap().len();
    assert!(
        run(
            &[
                "account",
                "create-account",
                "fund-later",
                "use-auto-generation",
                "save-to-keychain",
                "network-config",
                "testnet",
            ],
            context(&invalid_dir),
        )
        .is_err()
    );
    assert_eq!(store.entries.lock().unwrap().len(), entry_count);

    let missing_picker_dir = temp.path().join("missing-picker");
    std::fs::create_dir_all(missing_picker_dir.join("accounts.json")).unwrap();
    let err = run(
        &[
            "account",
            "create-account",
            "fund-later",
            "use-auto-generation",
            "save-to-keychain",
            "network-config",
            "testnet",
        ],
        context(&missing_picker_dir),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("could not be added to the account picker")
    );
    let saved_accounts = store
        .entries
        .lock()
        .unwrap()
        .iter()
        .filter_map(|((_, _), credential)| credential.get_password().ok())
        .map(|password| {
            serde_json::from_str::<near_cli_rs::common::KeyPairProperties>(&password)
                .unwrap()
                .implicit_account_id
        })
        .collect::<Vec<_>>();
    assert!(
        saved_accounts
            .iter()
            .any(|id| err.to_string().contains(id.as_str()))
    );
    assert!(near_cli_rs::common::get_used_account_list(&missing_picker_dir).is_empty());
    assert_eq!(std::fs::read_dir(&missing_picker_dir).unwrap().count(), 1);

    // The original explicit export command still writes the original schema,
    // and does not touch the secure credential store.
    let entry_count = store.entries.lock().unwrap().len();
    let export_dir = temp.path().join("export");
    run(
        &[
            "account",
            "create-account",
            "fund-later",
            "use-auto-generation",
            "save-to-folder",
            export_dir.to_str().unwrap(),
        ],
        global_context,
    )
    .unwrap();
    assert_eq!(store.entries.lock().unwrap().len(), entry_count);
    let files = std::fs::read_dir(export_dir).unwrap().collect::<Vec<_>>();
    assert_eq!(files.len(), 1);
    let exported: near_cli_rs::common::KeyPairProperties =
        serde_json::from_slice(&std::fs::read(files[0].as_ref().unwrap().path()).unwrap()).unwrap();
    assert!(!exported.master_seed_phrase.is_empty());
    assert_eq!(
        files[0].as_ref().unwrap().file_name(),
        format!("{}.json", exported.implicit_account_id).as_str(),
    );
}
