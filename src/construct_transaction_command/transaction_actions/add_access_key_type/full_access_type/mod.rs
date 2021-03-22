use async_recursion::async_recursion;
use std::str::FromStr;
use structopt::StructOpt;

use crate::construct_transaction_command::receiver::{CliSkipNextAction, NextAction};

#[derive(Debug)]
pub struct FullAccessType {
    pub next_action: Box<NextAction>,
}

#[derive(Debug, StructOpt)]
pub struct CliFullAccessType {
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliFullAccessType> for FullAccessType {
    fn from(item: CliFullAccessType) -> Self {
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => Box::new(NextAction::from(cli_skip_action)),
            None => Box::new(NextAction::input_next_action()),
        };
        FullAccessType { next_action }
    }
}

impl FullAccessType {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        public_key: near_crypto::PublicKey,
    ) {
        println!("FullAccessType process: self:\n       {:?}", &self);
        println!(
            "FullAccessType process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        // let public_key = near_crypto::PublicKey::from_str(&public_key_string).unwrap();
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
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
}
