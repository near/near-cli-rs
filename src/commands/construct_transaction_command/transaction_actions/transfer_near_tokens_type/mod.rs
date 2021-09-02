use async_recursion::async_recursion;
use dialoguer::Input;

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
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub next_action: Box<super::NextAction>,
}

impl CliTransferNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
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
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl TransferNEARTokensAction {
    pub fn from(
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
        let skip_next_action: super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                connection_config,
                sender_account_id,
            )?,
            None => super::NextAction::input_next_action(connection_config, sender_account_id)?,
        };
        Ok(Self {
            amount,
            next_action: Box::new(skip_next_action),
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

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: self.amount.to_yoctonear(),
            },
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
