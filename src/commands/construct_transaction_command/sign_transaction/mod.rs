use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod sign_manually;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
pub mod sign_with_private_key;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
///Would you like to sign the transaction?
pub enum SignTransaction {
    /// Provide arguments to sign a private key transaction
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with a plain-text private key"
    ))]
    SignPrivateKey(self::sign_with_private_key::SignPrivateKey),
    /// Provide arguments to sign a keychain transaction
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with keychain (located in ~/.near-credentials)"
    ))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    /// Connect your Ledger device and sign transaction with it
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with Ledger Nano S/X device"
    ))]
    SignWithLedger(self::sign_with_ledger::SignLedger),
    /// Provide arguments to sign a manually transaction
    #[strum_discriminants(strum(
        message = "No, I want to construct the transaction and sign it somewhere else"
    ))]
    SignManually(self::sign_manually::SignManually),
}

impl SignTransaction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            SignTransaction::SignPrivateKey(keys) => {
                keys.process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignWithKeychain(chain) => {
                chain
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            #[cfg(feature = "ledger")]
            SignTransaction::SignWithLedger(ledger) => {
                ledger
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignManually(args_manually) => {
                args_manually
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

fn input_signer_public_key() -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
    Ok(Input::new()
        .with_prompt("Enter sender (signer) public key")
        .interact_text()?)
}

fn input_signer_private_key() -> color_eyre::eyre::Result<crate::types::secret_key::SecretKey> {
    Ok(Input::new()
        .with_prompt("Enter sender (signer) private (secret) key")
        .interact_text()?)
}

fn input_access_key_nonce(public_key: &str) -> color_eyre::eyre::Result<u64> {
    println!("Your public key: `{}`", public_key);
    Ok(Input::new()
        .with_prompt(
            "Enter transaction nonce for this public key (query the access key information with \
            `./near-cli view nonce \
                network testnet \
                account 'volodymyr.testnet' \
                public-key ed25519:...` incremented by 1)",
        )
        .interact_text()?)
}

fn input_block_hash() -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
    let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
        .with_prompt(
            "Enter recent block hash (query information about the hash of the last block with \
            `./near-cli view recent-block-hash network testnet`)",
        )
        .interact_text()?;
    Ok(crate::types::crypto_hash::CryptoHash(
        input_block_hash.inner,
    ))
}

#[derive(Debug, EnumDiscriminants, Clone, clap::Clap, interactive_clap_derive::ToCliArgs)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(message = "I want to send the transaction to the network"))]
    Send,
    #[strum_discriminants(strum(
        message = "I only want to print base64-encoded transaction for JSON RPC input and exit"
    ))]
    Display,
}

impl Submit {
    pub fn choose_submit(connection_config: Option<crate::common::ConnectionConfig>) -> Self {
        if connection_config.is_none() {
            return Submit::Display;
        }
        println!();

        let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();
        let submits = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_submit = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("How would you like to proceed")
            .items(&submits)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_submit] {
            SubmitDiscriminants::Send => Submit::Send,
            SubmitDiscriminants::Display => Submit::Display,
        }
    }

    pub fn process_offline(
        self,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        println!("Serialize_to_base64:\n{}", &serialize_to_base64);
        Ok(None)
    }

    pub async fn process_online(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            Submit::Send => {
                println!("Transaction sent ...");
                let json_rcp_client = near_jsonrpc_client::JsonRpcClient::connect(
                    network_connection_config.rpc_url().as_str(),
                );
                let transaction_info = loop {
                    let transaction_info_result = json_rcp_client
                        .call(near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest{signed_transaction: signed_transaction.clone()})
                        .await;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => {
                            match err {
                                near_jsonrpc_client::errors::JsonRpcError::TransportError(_rpc_transport_error) => {
                                    println!("Transport error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                                }
                                near_jsonrpc_client::errors::JsonRpcError::ServerError(rpc_server_error) => match rpc_server_error {
                                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(rpc_transaction_error) => match rpc_transaction_error {
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::TimeoutError => {
                                            println!("Timeout error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                                        }
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::InvalidTransaction { context } => {
                                            crate::common::print_invalid_tx_error(context);
                                            return Ok(None);
                                        }
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::DoesNotTrackShard => {
                                            println!("RPC Server Error");
                                            return Ok(None)
                                        }
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::RequestRouted{transaction_hash} => {
                                            println!("RPC Server Error: {}", transaction_hash);
                                            return Ok(None)
                                        }
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::UnknownTransaction{requested_transaction_hash} => {
                                            println!("RPC Server Error: {}", requested_transaction_hash);
                                            return Ok(None)
                                        }
                                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::InternalError{debug_info} => {
                                            println!("RPC Server Error: {}", debug_info);
                                            return Ok(None)
                                        }
                                    }
                                    near_jsonrpc_client::errors::JsonRpcServerError::RequestValidationError(rpc_request_validation_error) => {
                                        println!("Incompatible request with the server: {:#?}",  rpc_request_validation_error);
                                        return Ok(None)
                                    }
                                    near_jsonrpc_client::errors::JsonRpcServerError::InternalError{ info } => {
                                        println!("Internal server error: {}.\nPlease wait. The next try to send this transaction is happening right now ...", info.unwrap_or_default());
                                    }
                                    near_jsonrpc_client::errors::JsonRpcServerError::NonContextualError(rpc_error) => {
                                        println!("Unexpected response: {}", rpc_error);
                                        return Ok(None)
                                    }
                                    near_jsonrpc_client::errors::JsonRpcServerError::ResponseStatusError(json_rpc_server_response_status_error) => match json_rpc_server_response_status_error {
                                        near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unauthorized => {
                                            println!("JSON RPC server requires authentication. Please, authenticate near CLI with the JSON RPC server you use.");
                                            return Ok(None)
                                        }
                                        near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::TooManyRequests => {
                                            println!("JSON RPC server is currently busy.\nPlease wait. The next try to send this transaction is happening right now ...");
                                        }
                                        near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unexpected{status} => {
                                            println!("JSON RPC server responded with an unexpected status code: {}", status);
                                            return Ok(None);
                                        }
                                    }
                                }
                            }
                            actix::clock::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    };
                };
                Ok(Some(transaction_info))
            }
            Submit::Display => {
                println!("\nSerialize_to_base64:\n{}", &serialize_to_base64);
                Ok(None)
            }
        }
    }
}
