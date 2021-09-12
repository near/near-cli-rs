use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliStake {
    /// Enter an amount
    Amount(CliStakeNEARTokensAction),
}

#[derive(Debug, Clone)]
pub enum Stake {
    Amount(StakeNEARTokensAction),
}

impl CliStake {
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

impl From<Stake> for CliStake {
    fn from(stake: Stake) -> Self {
        match stake {
            Stake::Amount(stake_near_tokens_action) => {
                Self::Amount(stake_near_tokens_action.into())
            }
        }
    }
}

impl Stake {
    pub fn from(
        item: CliStake,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliStake::Amount(cli_stake_near_action) => {
                Ok(Self::Amount(StakeNEARTokensAction::from(
                    cli_stake_near_action,
                    connection_config,
                    sender_account_id,
                )?))
            }
        }
    }
}

impl Stake {
    pub fn choose_stake_near(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliStake::Amount(Default::default()),
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
            Stake::Amount(transfer_near_action) => {
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
pub struct CliStakeNEARTokensAction {
    stake_amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    sign_transactions: Option<super::transactions_signing::CliTransactionsSigning>,
}

#[derive(Debug, Clone)]
pub struct StakeNEARTokensAction {
    pub stake_amount: crate::common::NearBalance,
    pub sign_transactions: super::transactions_signing::TransactionsSigning,
}

impl CliStakeNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_transactions
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(stake_amount) = &self.stake_amount {
            args.push_front(stake_amount.to_string());
        }
        args
    }
}

impl From<StakeNEARTokensAction> for CliStakeNEARTokensAction {
    fn from(stake_near_tokens_action: StakeNEARTokensAction) -> Self {
        Self {
            stake_amount: Some(stake_near_tokens_action.stake_amount.into()),
            sign_transactions: Some(stake_near_tokens_action.sign_transactions.into()),
        }
    }
}

impl StakeNEARTokensAction {
    fn from(
        item: CliStakeNEARTokensAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let stake_amount: crate::common::NearBalance = match item.stake_amount {
            Some(cli_stake_amount) => cli_stake_amount,
            None => StakeNEARTokensAction::input_stake_amount(),
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
            stake_amount,
            sign_transactions,
        })
    }
}

impl StakeNEARTokensAction {
    fn input_stake_amount() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
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
                self.stake_amount.to_yoctonear(),
            )
            .await
    }
}
