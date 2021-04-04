use async_recursion::async_recursion;
use clap::Clap;
use dialoguer::Input;

use crate::common::NearBalance;
use super::super::receiver::{CliSkipNextAction, CliNextAction, NextAction};

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: NearBalance,
    pub next_action: Box<NextAction>,
}

impl TransferNEARTokensAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let amount = match self.amount {
            NearBalance(num) => num,
        };
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction { deposit: amount },
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
    fn input_amount() -> NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
    }
}

#[derive(Debug, Default, Clap)]
pub struct CliTransferNEARTokensAction {
    amount: Option<NearBalance>,
    #[clap(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => TransferNEARTokensAction::input_amount(),
        };
        let cli_skip_next_action: CliNextAction = match item.next_action {
            Some(cli_skip_action) => CliNextAction::from(cli_skip_action),
            None => NextAction::input_next_action(),
        };
        TransferNEARTokensAction {
            amount,
            next_action: Box::new(NextAction::from(cli_skip_next_action)),
        }
    }
}
