use dialoguer::Input;

pub mod operation_mode;


#[derive(Debug, Default, clap::Clap)]
pub struct CliTransaction {
    transaction: Option<String>
}

#[derive(Debug)]
pub struct Transaction {
    transaction: String
}

impl From<CliTransaction> for Transaction {
    fn from(item: CliTransaction) -> Self {
        let transaction = match item.transaction {
            Some(transaction) => transaction,
            None => Transaction::input_transaction()
        };
        Self {transaction}
    }
}

impl Transaction {
    fn input_transaction() -> String {
        Input::new()
            .with_prompt("Enter the signed transaction hash you want to send")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self, network_connection_config: crate::common::ConnectionConfig) -> crate::CliResult {
        println!("\n\n\n--- Transaction sent ---");
        let json_rcp_client =
            near_jsonrpc_client::new_client(network_connection_config.rpc_url().as_str());
        let transaction_info = loop {
            let transaction_info_result = json_rcp_client
                .broadcast_tx_commit(self.transaction.clone())
                .await;
            match transaction_info_result {
                Ok(response) => {
                    break response;
                }
                Err(err) => {
                    if let Some(serde_json::Value::String(data)) = &err.data {
                        if data.contains("Timeout") {
                            println!("Error transaction: {:?}", err);
                            continue;
                        }
                    }
                    return Err(color_eyre::Report::msg(format!(
                        "Error transaction: {:?}",
                        err
                    )));
                }
            };
        };
        println!("\n\n--- Transaction execution: ---\n");
        match transaction_info.status {
            near_primitives::views::FinalExecutionStatus::NotStarted
            | near_primitives::views::FinalExecutionStatus::Started => unreachable!(),
            near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                crate::common::print_transaction_error(tx_execution_error).await
            }
            near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                for action in transaction_info.transaction.actions {
                    match action {
                        near_primitives::views::ActionView::CreateAccount => {
                            println!(
                                "\nNew account <{}> has been successfully created.",
                                transaction_info.transaction.receiver_id,
                            );
                        }
                        near_primitives::views::ActionView::DeployContract { code: _ } => {
                            println!("\n Contract code has been successfully deployed.",);
                        }
                        near_primitives::views::ActionView::FunctionCall {
                            method_name,
                            args: _,
                            gas: _,
                            deposit: _,
                        } => {
                            println!(
                                "\nThe \"{}\" call to <{}> on behalf of <{}> succeeded.",
                                method_name,
                                transaction_info.transaction.receiver_id,
                                transaction_info.transaction.signer_id,
                            );
                        }
                        near_primitives::views::ActionView::Transfer { deposit } => {
                            println!(
                                "\n<{}> has transferred {} to <{}> successfully.",
                                transaction_info.transaction.signer_id,
                                crate::common::NearBalance::from_yoctonear(deposit),
                                transaction_info.transaction.receiver_id,
                            );
                        }
                        near_primitives::views::ActionView::Stake {
                            stake,
                            public_key: _,
                        } => {
                            println!(
                                "\nValidator <{}> has successfully staked {}.",
                                transaction_info.transaction.signer_id,
                                crate::common::NearBalance::from_yoctonear(stake),
                            );
                        }
                        near_primitives::views::ActionView::AddKey {
                            public_key,
                            access_key: _,
                        } => {
                            println!(
                                "Added access key = {} to {}.",
                                public_key, transaction_info.transaction.receiver_id,
                            );
                        }
                        near_primitives::views::ActionView::DeleteKey { public_key } => {
                            println!(
                                "\nAccess key <{}> for account <{}> has been successfully deletted.",
                                public_key,
                                transaction_info.transaction.signer_id,
                            );
                        }
                        near_primitives::views::ActionView::DeleteAccount {
                            beneficiary_id: _,
                        } => {
                            println!(
                                "\nAccount <{}> has been successfully deletted.",
                                transaction_info.transaction.signer_id,
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
