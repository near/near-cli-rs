use async_recursion::async_recursion;
use dialoguer::Input;
use std::str::FromStr;
use structopt::StructOpt;

use super::super::receiver::{CliSkipNextAction, NextAction};

#[derive(Debug)]
pub struct DeleteAccessKeyAction {
    pub public_key: String,
    pub next_action: Box<NextAction>,
}

#[derive(Debug, StructOpt)]
pub struct CliDeleteAccessKeyAction {
    #[structopt(long)]
    public_key: Option<String>,
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliDeleteAccessKeyAction> for DeleteAccessKeyAction {
    fn from(item: CliDeleteAccessKeyAction) -> Self {
        let public_key: String = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyAction::input_public_key(),
        };
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => Box::new(NextAction::from(cli_skip_action)),
            None => Box::new(NextAction::input_next_action()),
        };
        DeleteAccessKeyAction {
            public_key,
            next_action,
        }
    }
}

impl DeleteAccessKeyAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("DeleteAccessKeyAction process: self:\n       {:?}", &self);
        println!(
            "DeleteAccessKeyAction process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        let public_key = near_crypto::PublicKey::from_str(&self.public_key).unwrap();
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction { public_key },
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
    pub fn input_public_key() -> String {
        Input::new()
            .with_prompt("Enter the access key to remove it")
            .interact_text()
            .unwrap()
    }
}
