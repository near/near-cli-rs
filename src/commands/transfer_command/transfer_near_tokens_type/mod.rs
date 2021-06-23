use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliTransfer {
    /// Enter an amount
    Amount(CliTransferNEARTokensAction),
}

#[derive(Debug)]
pub enum Transfer {
    Amount(TransferNEARTokensAction),
}

impl From<CliTransfer> for Transfer {
    fn from(item: CliTransfer) -> Self {
        match item {
            CliTransfer::Amount(cli_transfer_near_action) => {
                Self::Amount(cli_transfer_near_action.into())
            }
        }
    }
}

impl Transfer {
    pub fn choose_transfer_near() -> Self {
        Self::from(CliTransfer::Amount(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            Transfer::Amount(transfer_near_action) => {
                transfer_near_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// создание перевода токенов
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
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
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
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
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                )
                .await;
            }
            None => {}
        };
        Ok(())
    }
}
