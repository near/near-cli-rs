use super::*;
use clap::Parser;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};

// Only receiver queries are served. No signing or transaction submission is involved.
fn with_rpc<T>(
    response: Value,
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
            assert_eq!(request["method"], "query");
            assert_eq!(request["params"]["request_type"], "view_account");
            let mut response = response.clone();
            response["jsonrpc"] = json!("2.0");
            response["id"] = request["id"].clone();
            let response = response.to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
            requests.push(request);
        }
        requests
    });
    let mut config = crate::config::Config::default().network_connection["mainnet"].clone();
    config.rpc_url = format!("http://{address}").parse().unwrap();
    config.rpc_api_key = None;
    let result = run(&config);
    stop.send(()).unwrap();
    (result, server.join().unwrap())
}

fn exists() -> Value {
    json!({"result": {
        "amount": "1000000000000000000000000", "locked": "0", "storage_usage": 0,
        "code_hash": near_primitives::hash::CryptoHash::default(),
        "block_height": 1, "block_hash": near_primitives::hash::CryptoHash::default()
    }})
}

fn missing(receiver: &str) -> Value {
    json!({"error": {
        "code": -32000, "message": "Server error",
        "data": "account does not exist", "name": "HANDLER_ERROR",
        "cause": {"name": "UNKNOWN_ACCOUNT", "info": {
            "requested_account_id": receiver, "block_height": 1,
            "block_hash": near_primitives::hash::CryptoHash::default()
        }}
    }})
}

fn action_context(
    receiver: &str,
    choice: Option<ReceiverValidation>,
    verbosity: crate::Verbosity,
    offline: bool,
) -> crate::commands::ActionContext {
    SendNearCommandContext {
        global_context: crate::GlobalContext {
            config: crate::config::Config::default(),
            offline,
            verbosity,
        },
        signer_account_id: "sender.near".parse().unwrap(),
        receiver_account_id: receiver.parse().unwrap(),
        amount_in_near: "1 NEAR".parse().unwrap(),
        receiver_validation: choice,
        interactive: false,
    }
    .into()
}

#[test]
fn explicit_skip_and_legacy_quiet_make_no_receiver_requests() {
    let (_, requests) = with_rpc(exists(), |config| {
        for receiver in ["missing.near", "wrong.testnet", &"a".repeat(64)] {
            for offline in [false, true] {
                for verbosity in [crate::Verbosity::Interactive, crate::Verbosity::Quiet] {
                    let context = action_context(
                        receiver,
                        Some(ReceiverValidation::Skip),
                        verbosity,
                        offline,
                    );
                    let tx = (context.get_prepopulated_transaction_after_getting_network_callback)(
                        config,
                    )
                    .unwrap();
                    assert!(matches!(
                        tx.actions.as_slice(),
                        [near_primitives::transaction::Action::Transfer(_)]
                    ));
                }
                let context = action_context(receiver, None, crate::Verbosity::Quiet, offline);
                assert!(
                    (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                        .is_ok()
                );
            }
        }
    });
    assert!(requests.is_empty());
}

#[test]
fn explicit_check_queries_existing_receiver_even_when_quiet() {
    let (_, requests) = with_rpc(exists(), |config| {
        for choice in [None, Some(ReceiverValidation::Check)] {
            let context = action_context(
                "receiver.near",
                choice,
                crate::Verbosity::Interactive,
                false,
            );
            assert!(
                (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                    .is_ok()
            );
        }
        let context = action_context(
            "receiver.near",
            Some(ReceiverValidation::Check),
            crate::Verbosity::Quiet,
            false,
        );
        assert!(
            (context.get_prepopulated_transaction_after_getting_network_callback)(config).is_ok()
        );
    });
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|r| r["params"]["account_id"] == "receiver.near")
    );
}

#[test]
fn scripted_check_rejects_missing_named_and_fresh_implicit_without_dialogs() {
    for receiver in ["missing.near".to_string(), "a".repeat(64)] {
        let (_, requests) = with_rpc(missing(&receiver), |config| {
            for verbosity in [crate::Verbosity::Interactive, crate::Verbosity::Quiet] {
                let context =
                    action_context(&receiver, Some(ReceiverValidation::Check), verbosity, false);
                let err =
                    (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                        .unwrap_err();
                assert!(err.to_string().contains("does not exist"), "{err}");
                assert!(err.to_string().contains("--receiver-validation skip"));
            }
            // The existing interactive warning still lets the user cancel or proceed.
            for proceed in [false, true] {
                let result = crate::common::validate_receiver_account_id_with_warning(
                    config,
                    &receiver.parse().unwrap(),
                    crate::Verbosity::Interactive,
                    false,
                    |message| {
                        assert!(message.contains("does not exist"));
                        Ok(proceed)
                    },
                )
                .unwrap();
                assert_eq!(result, proceed);
            }
        });
        assert_eq!(requests.len(), 4);
    }
}

#[test]
fn offline_defaults_keep_warning_and_explicit_check_cannot_use_rpc() {
    let (_, requests) = with_rpc(exists(), |config| {
        let receiver = "receiver.near".parse().unwrap();
        let result = crate::common::validate_receiver_account_id_with_warning(
            config,
            &receiver,
            crate::Verbosity::Interactive,
            true,
            |message| {
                assert!(message.contains("offline mode"));
                Ok(false)
            },
        )
        .unwrap();
        assert!(!result);
        // Quiet still bypasses the offline warning.
        assert!(
            crate::common::validate_receiver_account_id_with_warning(
                config,
                &receiver,
                crate::Verbosity::Quiet,
                true,
                |_| panic!("quiet must not prompt")
            )
            .unwrap()
        );
        for verbosity in [crate::Verbosity::Interactive, crate::Verbosity::Quiet] {
            let context = action_context(
                "receiver.near",
                Some(ReceiverValidation::Check),
                verbosity,
                true,
            );
            let err = (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                .unwrap_err();
            assert!(
                err.to_string()
                    .contains("Cannot check the receiver account in offline mode")
            );
        }
    });
    assert!(requests.is_empty());
}

#[test]
fn named_account_network_safeguard_is_preserved_when_checking() {
    let (_, requests) = with_rpc(exists(), |config| {
        let context = action_context(
            "wrong.testnet",
            Some(ReceiverValidation::Check),
            crate::Verbosity::Quiet,
            false,
        );
        let err = (context.get_prepopulated_transaction_after_getting_network_callback)(config)
            .unwrap_err();
        assert!(err.to_string().contains("different network"));
        assert!(
            !crate::common::validate_receiver_account_id_with_warning(
                config,
                &"wrong.testnet".parse().unwrap(),
                crate::Verbosity::Interactive,
                false,
                |message| {
                    assert!(message.contains("different network"));
                    Ok(false)
                }
            )
            .unwrap()
        );
    });
    assert!(requests.is_empty());
}

#[test]
fn rpc_errors_propagate_instead_of_becoming_missing_receiver_warnings() {
    let (_, requests) = with_rpc(
        json!({"error": {"code": -32603, "message": "Internal error", "data": "unavailable"}}),
        |config| {
            let context = action_context(
                "receiver.near",
                Some(ReceiverValidation::Check),
                crate::Verbosity::Quiet,
                false,
            );
            let err = (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                .unwrap_err();
            assert!(!err.to_string().contains("does not exist"));
        },
    );
    assert_eq!(requests.len(), 6); // Existing retry policy is unchanged.
    let (_, requests) = with_rpc(
        json!({"result": {"values": [], "proof": [], "block_height": 1, "block_hash": near_primitives::hash::CryptoHash::default()}}),
        |config| {
            let context = action_context(
                "receiver.near",
                Some(ReceiverValidation::Check),
                crate::Verbosity::Interactive,
                false,
            );
            let err = (context.get_prepopulated_transaction_after_getting_network_callback)(config)
                .unwrap_err();
            assert!(
                err.to_string().contains("unexpected server response"),
                "{err}"
            );
        },
    );
    assert_eq!(requests.len(), 1);
}

#[test]
fn cli_validation_option_is_optional_and_round_trips() {
    use interactive_clap::ToCliArgs;
    for choice in [None, Some("skip"), Some("check")] {
        let mut args = vec!["send-near", "receiver.near", "1 NEAR"];
        if let Some(choice) = choice {
            args.extend(["--receiver-validation", choice]);
        }
        args.extend(["network-config", "mainnet"]);
        let cli = CliSendNearCommand::try_parse_from(&args).unwrap();
        assert_eq!(
            cli.receiver_validation.map(|v| v.to_string()),
            choice.map(str::to_owned)
        );
        let mut round_trip = vec!["send-near".to_string()];
        round_trip.extend(cli.to_cli_args());
        let cli_again = CliSendNearCommand::try_parse_from(round_trip).unwrap();
        assert_eq!(cli_again.receiver_validation, cli.receiver_validation);
    }
    assert!(
        CliSendNearCommand::try_parse_from([
            "send-near",
            "receiver.near",
            "1 NEAR",
            "--receiver-validation",
            "invalid"
        ])
        .is_err()
    );
}

#[test]
fn scripted_from_cli_check_stops_before_signing_and_preserves_flag() {
    let ((), requests) = with_rpc(missing("missing.near"), |config| {
        let mut global = crate::GlobalContext {
            config: crate::config::Config::default(),
            verbosity: crate::Verbosity::Quiet,
            offline: false,
        };
        global
            .config
            .network_connection
            .insert("mock".to_owned(), config.clone());
        let cli = CliSendNearCommand::try_parse_from([
            "send-near",
            "missing.near",
            "1 NEAR",
            "--receiver-validation",
            "check",
            "network-config",
            "mock",
            "sign-with-keychain",
            "send",
        ])
        .unwrap();
        let result = <SendNearCommand as interactive_clap::FromCli>::from_cli(
            Some(cli),
            super::super::TokensCommandsContext {
                global_context: global,
                owner_account_id: "sender.near".parse().unwrap(),
            },
        );
        match result {
            interactive_clap::ResultFromCli::Err(Some(cli), err) => {
                assert_eq!(cli.receiver_validation, Some(ReceiverValidation::Check));
                assert!(err.to_string().contains("does not exist"));
            }
            _ => panic!("check should fail before reaching signing options"),
        }
    });
    assert_eq!(requests.len(), 1);
}

#[test]
fn complete_scripts_and_partial_interactive_commands_are_distinguished() {
    let prefix = ["send-near", "receiver.near", "1 NEAR"];
    let partial: &[&[&str]] = &[
        &[],
        &["network-config"],
        &["network-config", "mainnet"],
        &["network-config", "mainnet", "sign-with-keychain"],
        &["network-config", "mainnet", "sign-with-legacy-keychain"],
        &["network-config", "mainnet", "sign-with-access-key-file"],
        &[
            "network-config",
            "mainnet",
            "sign-with-access-key-file",
            "key.json",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-access-key-file",
            "key.json",
            "save-to-file",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-plaintext-private-key",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-seed-phrase",
            "test seed",
            "send",
        ],
        &["network-config", "mainnet", "sign-later", "display"],
        &["network-config", "mainnet", "sign-with-mpc"],
        &["network-config", "mainnet", "submit-as-dao-proposal"],
    ];
    for tail in partial {
        let cli = CliSendNearCommand::try_parse_from(prefix.iter().chain(tail.iter())).unwrap();
        assert!(needs_interactive_input(&cli), "partial tail: {tail:?}");
    }
    let complete: &[&[&str]] = &[
        &[
            "network-config",
            "mainnet",
            "sign-later",
            "--signer-public-key",
            "ed25519:11111111111111111111111111111111",
            "--nonce",
            "1",
            "--block-hash",
            "11111111111111111111111111111111",
            "display",
        ],
        &[
            "network-config",
            "mainnet",
            "submit-as-dao-proposal",
            "dao.near",
            "transfer proposal",
            "prepaid-gas",
            "10 Tgas",
            "attached-deposit",
            "1 NEAR",
            "sign-with-keychain",
            "send",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-mpc",
            "admin.near",
            "ed25519",
            "derivation-path",
            "test path",
            "prepaid-gas",
            "10 Tgas",
            "attached-deposit",
            "1 NEAR",
            "sign-mpc-with-keychain",
            "send",
        ],
        &["network-config", "mainnet", "sign-with-keychain", "send"],
        &["network-config", "mainnet", "sign-with-keychain", "display"],
        &[
            "network-config",
            "mainnet",
            "sign-with-keychain",
            "save-to-file",
            "signed.json",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-access-key-file",
            "key.json",
            "send",
        ],
        &[
            "network-config",
            "mainnet",
            "sign-with-seed-phrase",
            "test seed",
            "--seed-phrase-hd-path",
            "m/44'/397'/0'",
            "send",
        ],
    ];
    for tail in complete {
        let cli = CliSendNearCommand::try_parse_from(prefix.iter().chain(tail.iter())).unwrap();
        assert!(!needs_interactive_input(&cli), "complete tail: {tail:?}");
    }
    #[cfg(feature = "ledger")]
    {
        let cli = CliSendNearCommand::try_parse_from(
            prefix.iter().chain(
                [
                    "network-config",
                    "mainnet",
                    "sign-with-ledger",
                    "usb",
                    "send",
                ]
                .iter(),
            ),
        )
        .unwrap();
        assert!(needs_interactive_input(&cli));
        let cli = CliSendNearCommand::try_parse_from(
            prefix.iter().chain(
                [
                    "network-config",
                    "mainnet",
                    "sign-with-ledger",
                    "--seed-phrase-hd-path",
                    "44'/397'/0'/0'/1'",
                    "usb",
                    "send",
                ]
                .iter(),
            ),
        )
        .unwrap();
        assert!(!needs_interactive_input(&cli));
    }
}
