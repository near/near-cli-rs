use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

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
    pub amount: crate::common::TransferAmount,
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
            amount: Some(transfer_near_tokens_action.amount.into()),
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
        let amount: crate::common::TransferAmount = match item.amount {
            Some(cli_amount) => crate::common::TransferAmount::from_unchecked(cli_amount),
            None => TransferNEARTokensAction::input_amount(
                connection_config.clone(),
                sender_account_id.clone(),
            )?,
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
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<crate::common::TransferAmount> {
        match connection_config {
            Some(connection_config) => {
                let account_transfer_allowance = crate::common::get_account_transfer_allowance(
                    &connection_config,
                    sender_account_id,
                )?;
                loop {
                    let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        .unwrap();
                    if let Ok(transfer_amount) = crate::common::TransferAmount::from(
                        input_amount.clone(),
                        &account_transfer_allowance,
                    ) {
                        break Ok(transfer_amount);
                    } else {
                        println!(
                            "\nWARNING! There is only {} available for transfer.",
                            account_transfer_allowance.transfer_allowance()
                        );
                        let choose_input = vec![
                            format!("Yes, I'd like to transfer {}.", input_amount),
                            "No, I'd like to change the transfer amount.".to_string(),
                        ];
                        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Do you want to keep this amount for the transfer?")
                            .items(&choose_input)
                            .default(0)
                            .interact_on_opt(&Term::stderr())
                            .unwrap();
                        match select_choose_input {
                            Some(0) => {
                                break Ok(crate::common::TransferAmount::from_unchecked(
                                    input_amount,
                                ))
                            }
                            Some(1) => {}
                            _ => unreachable!("Error"),
                        }
                    }
                }
            }
            None => {
                let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        .unwrap();
                Ok(crate::common::TransferAmount::from_unchecked(input_amount))
            }
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
