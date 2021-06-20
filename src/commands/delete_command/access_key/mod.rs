use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, clap::Clap)]
pub enum CliDeleteAccessKeyAction {
    /// Specify public key
    PublicKey(CliDeleteAccessKeyType),
}

#[derive(Debug)]
pub enum DeleteAccessKeyAction {
    PublicKey(DeleteAccessKeyType),
}

impl From<CliDeleteAccessKeyAction> for DeleteAccessKeyAction {
    fn from(item: CliDeleteAccessKeyAction) -> Self {
        match item {
            CliDeleteAccessKeyAction::PublicKey(cli_delete_access_key_type) => {
                Self::PublicKey(cli_delete_access_key_type.into())
            }
        }
    }
}

impl DeleteAccessKeyAction {
    pub fn choose_delete_access_key_action() -> Self {
        Self::from(CliDeleteAccessKeyAction::PublicKey(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            DeleteAccessKeyAction::PublicKey(delete_access_key_type) => {
                delete_access_key_type
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// Specify the access key to be deleted
#[derive(Debug, Default, clap::Clap)]
pub struct CliDeleteAccessKeyType {
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct DeleteAccessKeyType {
    pub public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliDeleteAccessKeyType> for DeleteAccessKeyType {
    fn from(item: CliDeleteAccessKeyType) -> Self {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyType::input_public_key(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            public_key,
            sign_option,
        }
    }
}

impl DeleteAccessKeyType {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key: self.public_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config.clone())
            .await?
        {
            Some(transaction_info) => {
                match transaction_info.status {
                    near_primitives::views::FinalExecutionStatus::NotStarted
                    | near_primitives::views::FinalExecutionStatus::Started => unreachable!(),
                    near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                        crate::common::print_transaction_error(tx_execution_error).await
                    }
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                        match transaction_info.transaction.actions[0].clone() {
                            near_primitives::views::ActionView::DeleteKey { public_key } => {
                                println!(
                                    "\nAccess key <{}> for account <{}> has been successfully deletted.",
                                    public_key,
                                    transaction_info.transaction.signer_id,
                                );
                            }
                            _ => unreachable!("Error"),
                        }
                    }
                }
                let transaction_explorer: url::Url = match network_connection_config {
                    Some(connection_config) => connection_config.transaction_explorer(),
                    None => unreachable!("Error"),
                };
                println!("\nTransaction Id {id}.\n\nTo see the transaction in the transaction explorer, please open this url in your browser:
                    \n{path}{id}\n", id=transaction_info.transaction_outcome.id, path=transaction_explorer);
            }
            None => {}
        };
        Ok(())
    }
}
