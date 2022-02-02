use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

pub mod operation_mode;

#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliTransaction {
    signed_transaction: Option<crate::common::SignedTransactionAsBase64>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    signed_transaction: near_primitives::transaction::SignedTransaction,
}

impl CliTransaction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(signed_transaction) = &self.signed_transaction {
            let signed_transaction_serialized_to_base64 = near_primitives::serialize::to_base64(
                signed_transaction
                    .inner
                    .try_to_vec()
                    .expect("Transaction is not expected to fail on serialization"),
            );
            args.push_front(signed_transaction_serialized_to_base64);
        }
        args
    }
}

impl From<Transaction> for CliTransaction {
    fn from(transaction: Transaction) -> Self {
        Self {
            signed_transaction: Some(crate::common::SignedTransactionAsBase64 {
                inner: transaction.signed_transaction,
            }),
        }
    }
}

impl From<CliTransaction> for Transaction {
    fn from(item: CliTransaction) -> Self {
        let signed_transaction = match item.signed_transaction {
            Some(cli_transaction) => cli_transaction.inner,
            None => Transaction::input_transaction(),
        };
        Self { signed_transaction }
    }
}

impl Transaction {
    fn input_transaction() -> near_primitives::transaction::SignedTransaction {
        let input: crate::common::SignedTransactionAsBase64 = Input::new()
            .with_prompt("Enter the signed transaction hash you want to send")
            .interact_text()
            .unwrap();
        input.inner
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!("Transaction sent ...");
        let json_rcp_client = near_jsonrpc_client::JsonRpcClient::connect(
            network_connection_config.rpc_url().as_str(),
        );
        let transaction_info = loop {
            let transaction_info_result = json_rcp_client
                .call(near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                    signed_transaction: self.signed_transaction.clone()
                })
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
                                    return Ok(());
                                }
                                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::DoesNotTrackShard => {
                                    println!("RPC Server Error");
                                    return Ok(())
                                }
                                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::RequestRouted{transaction_hash} => {
                                    println!("RPC Server Error: {}", transaction_hash);
                                    return Ok(())
                                }
                                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::UnknownTransaction{requested_transaction_hash} => {
                                    println!("RPC Server Error: {}", requested_transaction_hash);
                                    return Ok(())
                                }
                                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::InternalError{debug_info} => {
                                    println!("RPC Server Error: {}", debug_info);
                                    return Ok(())
                                }
                            }
                            near_jsonrpc_client::errors::JsonRpcServerError::RequestValidationError(rpc_request_validation_error) => {
                                println!("Incompatible request with the server: {:#?}",  rpc_request_validation_error);
                                return Ok(())
                            }
                            near_jsonrpc_client::errors::JsonRpcServerError::InternalError{ info } => {
                                println!("Internal server error: {}.\nPlease wait. The next try to send this transaction is happening right now ...", info.unwrap_or_default());
                            }
                            near_jsonrpc_client::errors::JsonRpcServerError::NonContextualError(rpc_error) => {
                                println!("Unexpected response: {}", rpc_error);
                                return Ok(())
                            }
                            near_jsonrpc_client::errors::JsonRpcServerError::ResponseStatusError(json_rpc_server_response_status_error) => match json_rpc_server_response_status_error {
                                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unauthorized => {
                                    println!("JSON RPC server requires authentication. Please, authenticate near CLI with the JSON RPC server you use.");
                                    return Ok(())
                                }
                                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::TooManyRequests => {
                                    println!("JSON RPC server is currently busy.\nPlease wait. The next try to send this transaction is happening right now ...");
                                }
                                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unexpected{status} => {
                                    println!("JSON RPC server responded with an unexpected status code: {}", status);
                                    return Ok(());
                                }
                            }
                        }
                    }
                    actix::clock::sleep(std::time::Duration::from_millis(100)).await;
                }
            };
        };
        crate::common::print_transaction_status(transaction_info, Some(network_connection_config));
        Ok(())
    }
}
