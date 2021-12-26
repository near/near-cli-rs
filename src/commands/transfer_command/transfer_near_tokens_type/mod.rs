use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use strum::EnumDiscriminants;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
#[interactive_clap(disable_strum_discriminants)]
pub enum Transfer {
    /// Enter an amount to transfer
    Amount(TransferNEARTokensAction),
}

impl Transfer {
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
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::TransferAmount,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl interactive_clap::ToCli for TransferNEARTokensAction {
    type CliVariant = CliTransferNEARTokensAction;
}

impl CliTransferNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_option
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
            sign_option: Some(transfer_near_tokens_action.sign_option.into()),
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

        let sign_option = match optional_clap_variant
        .clone()
        .and_then(|clap_variant| clap_variant.sign_option)
    {
        Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from_cli(Some(cli_sign_transaction), context)?,
        None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_variant(context)?,
    };

        Ok(Self {
            amount,
            sign_option,
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
            Some(connection_config) => {
                let account_transfer_allowance = crate::common::get_account_transfer_allowance(
                    &connection_config,
                    sender_account_id.into(),
                )?;
                println! {"{}", &account_transfer_allowance};
                loop {
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
                                break Ok(crate::common::TransferAmount::from_unchecked(
                                    input_amount,
                                ));
                            }
                            Some(1) => continue,
                            _ => unreachable!("Error"),
                        }
                    }
                }
            }
            None => {
                let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        ?;
                Ok(crate::common::TransferAmount::from_unchecked(input_amount))
            }
        }
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
                );
            }
            None => {}
        };
        Ok(())
    }
}
