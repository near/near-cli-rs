use async_recursion::async_recursion;
use clap::Clap;
use dialoguer::Input;

use crate::common::NearBalance;
use super::super::receiver::{CliSkipNextAction, CliNextAction, NextAction};

#[derive(Debug)]
pub struct StakeNEARTokensAction {
    pub stake: NearBalance,
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<NextAction>,
}

impl StakeNEARTokensAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let stake = match self.stake {
            NearBalance(num) => num,
        };
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake,
                public_key: self.public_key.clone()
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
            NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this stake")
            .interact_text()
            .unwrap()
    }
    fn input_stake() -> NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake?")
            .interact_text()
            .unwrap()
    }
}

#[derive(Debug, Default, Clap)]
pub struct CliStakeNEARTokensAction {
    stake: Option<NearBalance>,
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliStakeNEARTokensAction> for StakeNEARTokensAction {
    fn from(item: CliStakeNEARTokensAction) -> Self {
        let stake: NearBalance = match item.stake {
            Some(cli_stake) => cli_stake,
            None => StakeNEARTokensAction::input_stake(),
        };
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => StakeNEARTokensAction::input_public_key(),
        };
        let cli_skip_next_action: CliNextAction = match item.next_action {
            Some(cli_skip_action) => CliNextAction::from(cli_skip_action),
            None => NextAction::input_next_action(),
        };
        StakeNEARTokensAction {
            stake,
            public_key,
            next_action: Box::new(NextAction::from(cli_skip_next_action)),
        }
    }
}
