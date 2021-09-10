use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

/// создание ставки
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliStakeNEARTokensAction {
    stake: Option<crate::common::NearBalance>,
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct StakeNEARTokensAction {
    pub stake: crate::common::TransferAmount,
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<super::NextAction>,
}

impl CliStakeNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        };
        if let Some(stake) = &self.stake {
            args.push_front(stake.to_string());
        };
        args
    }
}

impl From<StakeNEARTokensAction> for CliStakeNEARTokensAction {
    fn from(stake_near_tokens_action: StakeNEARTokensAction) -> Self {
        Self {
            stake: Some(stake_near_tokens_action.stake.into()),
            public_key: Some(stake_near_tokens_action.public_key),
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl StakeNEARTokensAction {
    pub fn from(
        item: CliStakeNEARTokensAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let stake: crate::common::TransferAmount = match item.stake {
            Some(cli_stake) => crate::common::TransferAmount::from_unchecked(cli_stake),
            None => StakeNEARTokensAction::input_stake(
                connection_config.clone(),
                sender_account_id.clone(),
            )?,
        };
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => StakeNEARTokensAction::input_public_key(),
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
            stake,
            public_key,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl StakeNEARTokensAction {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this stake")
            .interact_text()
            .unwrap()
    }

    fn input_stake(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<crate::common::TransferAmount> {
        match connection_config {
            Some(connection_config) => {
                let account_transfer_allowance = crate::common::get_account_transfer_allowance(
                    &connection_config,
                    sender_account_id,
                )?;
                println! {"{}", &account_transfer_allowance};
                loop {
                    let input_amount: crate::common::NearBalance = Input::new()
                        .with_prompt("How many NEAR Tokens do you want to stake? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        .unwrap();
                    if let Ok(transfer_amount) = crate::common::TransferAmount::from(
                        input_amount.clone(),
                        &account_transfer_allowance,
                    ) {
                        break Ok(transfer_amount);
                    } else {
                        println!(
                            "\nWARNING! There is only {} available for stake.",
                            account_transfer_allowance.transfer_allowance()
                        );
                        let choose_input = vec![
                            format!("Yes, I'd like to stake {}.", input_amount),
                            "No, I'd like to change change the stake amount.".to_string(),
                        ];
                        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Do you want to keep this amount for the stake?")
                            .items(&choose_input)
                            .default(0)
                            .interact_on_opt(&Term::stderr())
                            .unwrap();
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
                        .with_prompt("How many NEAR Tokens do you want to stake? (example: 10NEAR or 0.5near or 10000yoctonear)")
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
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake: self.stake.to_yoctonear(),
                public_key: self.public_key.clone(),
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
