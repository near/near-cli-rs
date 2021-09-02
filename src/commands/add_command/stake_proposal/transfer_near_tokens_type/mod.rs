use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTransfer {
    /// Enter an amount
    Amount(CliTransferNEARTokensAction),
}

#[derive(Debug, Clone)]
pub enum Transfer {
    Amount(TransferNEARTokensAction),
}

impl CliTransfer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Amount(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("amount".to_owned());
                args
            }
        }
    }
}

impl From<Transfer> for CliTransfer {
    fn from(transfer: Transfer) -> Self {
        match transfer {
            Transfer::Amount(transfer_near_tokens_action) => {
                Self::Amount(transfer_near_tokens_action.into())
            }
        }
    }
}

impl Transfer {
    pub fn from(
        item: CliTransfer,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliTransfer::Amount(cli_transfer_near_action) => {
                Ok(Self::Amount(TransferNEARTokensAction::from(
                    cli_transfer_near_action,
                    connection_config,
                    sender_account_id,
                )?))
            }
        }
    }
}

impl Transfer {
    pub fn choose_transfer_near(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliTransfer::Amount(Default::default()),
            connection_config,
            sender_account_id,
        )?)
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
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliTransferNEARTokensAction {
    amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    sign_transactions: Option<super::transactions_signing::CliTransactionsSigning>,
}

#[derive(Debug, Clone)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub sign_transactions: super::transactions_signing::TransactionsSigning,
}

impl CliTransferNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_transactions
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(amount) = &self.amount {
            args.push_front(amount.to_string());
        }
        args
    }
}

impl From<TransferNEARTokensAction> for CliTransferNEARTokensAction {
    fn from(transfer_near_tokens_action: TransferNEARTokensAction) -> Self {
        Self {
            amount: Some(transfer_near_tokens_action.amount),
            sign_transactions: Some(transfer_near_tokens_action.sign_transactions.into()),
        }
    }
}

impl TransferNEARTokensAction {
    fn from(
        item: CliTransferNEARTokensAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let amount: crate::common::NearBalance = match &connection_config {
            Some(network_connection_config) => {
                let account_balance: crate::common::NearBalance =
                    match crate::common::check_account_id(
                        network_connection_config.clone(),
                        sender_account_id.clone(),
                    )? {
                        Some(account_view) => {
                            crate::common::NearBalance::from_yoctonear(account_view.amount)
                        }
                        None => crate::common::NearBalance::from_yoctonear(0),
                    };
                let max_allowable_transfer_amount: crate::common::NearBalance =
                    crate::common::get_max_allowable_transfer_amount(
                        network_connection_config.clone(),
                        sender_account_id.clone(),
                    )?;
                match item.amount {
                    Some(cli_amount) => {
                        if cli_amount <= max_allowable_transfer_amount {
                            cli_amount
                        } else {
                            println!(
                                "You need to enter a value of no more than {}",
                                max_allowable_transfer_amount
                            );
                            TransferNEARTokensAction::input_amount(
                                Some(account_balance),
                                max_allowable_transfer_amount,
                            )
                        }
                    }
                    None => TransferNEARTokensAction::input_amount(
                        Some(account_balance),
                        max_allowable_transfer_amount,
                    ),
                }
            }
            None => match item.amount {
                Some(cli_amount) => cli_amount,
                None => TransferNEARTokensAction::input_amount(
                    None,
                    crate::common::NearBalance::from_yoctonear(0),
                ),
            },
        };
        let sign_transactions = match item.sign_transactions {
            Some(cli_transaction_signing) => {
                super::transactions_signing::TransactionsSigning::from(
                    cli_transaction_signing,
                    connection_config,
                    sender_account_id,
                )?
            }
            None => super::transactions_signing::TransactionsSigning::choose_sign_transactions(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            amount,
            sign_transactions,
        })
    }
}

impl TransferNEARTokensAction {
    fn input_amount(
        account_balance: Option<crate::common::NearBalance>,
        max_allowable_transfer_amount: crate::common::NearBalance,
    ) -> crate::common::NearBalance {
        match account_balance {
            Some(account_balance) => loop {
                let input_amount: crate::common::NearBalance = Input::new()
                    .with_prompt(format!("On your account {}. How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)", account_balance))
                    .interact_text()
                    .unwrap();
                if input_amount <= max_allowable_transfer_amount {
                    break input_amount;
                } else {
                    println!(
                        "You need to enter a value of no more than {}",
                        max_allowable_transfer_amount
                    )
                }
            }
            None => Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        .unwrap()
        }
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.sign_transactions
            .process(
                prepopulated_unsigned_transaction,
                network_connection_config,
                self.amount.to_yoctonear(),
            )
            .await
    }
}
