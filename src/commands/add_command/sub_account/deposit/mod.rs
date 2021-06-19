use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliDeposit {
    /// Enter an amount
    Deposit(CliTransferNEARTokensAction),
}

#[derive(Debug)]
pub enum Deposit {
    Deposit(TransferNEARTokensAction),
}

impl From<CliDeposit> for Deposit {
    fn from(item: CliDeposit) -> Self {
        match item {
            CliDeposit::Deposit(cli_transfer_near_action) => {
                Self::Deposit(cli_transfer_near_action.into())
            }
        }
    }
}

impl Deposit {
    pub fn choose_deposit() -> Self {
        Self::from(CliDeposit::Deposit(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            Deposit::Deposit(transfer_near_action) => {
                transfer_near_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// создание перевода токенов
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransferNEARTokensAction {
    amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: crate::common::NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => TransferNEARTokensAction::input_amount(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            amount,
            sign_option,
        }
    }
}

impl TransferNEARTokensAction {
    pub fn input_amount() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to deposit? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: self.amount.to_yoctonear(),
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
                        println!(
                            "\nNew account <{}> has been successfully created.",
                            transaction_info.transaction.receiver_id,
                        );
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
