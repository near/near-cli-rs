use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliTransactionsSigning {
    /// Enter an public key
    TransactionsSigningPublicKey(CliTransactionsSigningAction),
}

#[derive(Debug)]
pub enum TransactionsSigning {
    TransactionsSigningPublicKey(TransactionsSigningAction),
}

impl From<CliTransactionsSigning> for TransactionsSigning {
    fn from(item: CliTransactionsSigning) -> Self {
        match item {
            CliTransactionsSigning::TransactionsSigningPublicKey(
                cli_transactions_signing_action,
            ) => Self::TransactionsSigningPublicKey(cli_transactions_signing_action.into()),
        }
    }
}

impl TransactionsSigning {
    pub fn choose_sign_transactions() -> Self {
        Self::from(CliTransactionsSigning::TransactionsSigningPublicKey(
            Default::default(),
        ))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        match self {
            TransactionsSigning::TransactionsSigningPublicKey(transactions_sign_action) => {
                transactions_sign_action
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        stake,
                    )
                    .await
            }
        }
    }
}

/// данные о получателе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransactionsSigningAction {
    transactions_signing_public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct TransactionsSigningAction {
    pub transactions_signing_public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliTransactionsSigningAction> for TransactionsSigningAction {
    fn from(item: CliTransactionsSigningAction) -> Self {
        let transactions_signing_public_key: near_crypto::PublicKey =
            match item.transactions_signing_public_key {
                Some(cli_transactions_signing_public_key) => cli_transactions_signing_public_key,
                None => TransactionsSigningAction::input_public_key(),
            };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            transactions_signing_public_key,
            sign_option,
        }
    }
}

impl TransactionsSigningAction {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this server")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake,
                public_key: self.transactions_signing_public_key,
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
            .process(unsigned_transaction, network_connection_config)
            .await?
        {
            Some(transaction_info) => {
                match transaction_info.status {
                    near_primitives::views::FinalExecutionStatus::NotStarted => {
                        println!("NotStarted")
                    }
                    near_primitives::views::FinalExecutionStatus::Started => println!("Started"),
                    near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                        crate::common::print_transaction_error(tx_execution_error).await
                    }
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                        match transaction_info.transaction.actions[0] {
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
                            _ => unreachable!("Error")
                        }
                    }
                }
                println!("\nTransaction Id {id}.\n\nTo see the transaction in the transaction explorer, please open this url in your browser:
                    \nhttps://explorer.testnet.near.org/transactions/{id}\n", id=transaction_info.transaction_outcome.id);
            }
            None => {}
        };
        Ok(())
    }
}
