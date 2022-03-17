use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

/// creating a transfer of tokens
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

impl interactive_clap::ToCli for TransferNEARTokensAction {
    type CliVariant = CliTransferNEARTokensAction;
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
    pub fn from_cli(
        optional_clap_variant: Option<
            <TransferNEARTokensAction as interactive_clap::ToCli>::CliVariant,
        >,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let amount: crate::common::TransferAmount = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.amount)
        {
            Some(cli_amount) => crate::common::TransferAmount::from_unchecked(cli_amount),
            None => TransferNEARTokensAction::input_amount(&context)?,
        };
        let skip_next_action: super::NextAction = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.next_action)
        {
            Some(cli_skip_action) => {
                super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
            }
            None => super::NextAction::choose_variant(context)?,
        };
        Ok(Self {
            amount,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl TransferNEARTokensAction {
    fn input_amount(
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::common::TransferAmount> {
        let connection_config = context.connection_config.clone();
        let sender_account_id = context.signer_account_id.clone();
        match connection_config {
            Some(connection_config) => loop {
                let account_transfer_allowance = crate::common::get_account_transfer_allowance(
                    &connection_config,
                    sender_account_id.clone().into(),
                )?;
                println! {"{}", &account_transfer_allowance};
                let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        ?;
                if let Ok(transfer_amount) = crate::common::TransferAmount::from(
                    input_amount.clone(),
                    &account_transfer_allowance,
                ) {
                    break Ok(transfer_amount);
                } else {
                    let account_transfer_allowance = crate::common::get_account_transfer_allowance(
                        &connection_config,
                        sender_account_id.clone().into(),
                    )?;
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
                        .interact_on_opt(&Term::stderr())?;
                    match select_choose_input {
                        Some(0) => {
                            break Ok(crate::common::TransferAmount::from_unchecked(input_amount))
                        }
                        Some(1) => {}
                        _ => unreachable!("Error"),
                    }
                }
            },
            None => {
                let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        ?;
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
