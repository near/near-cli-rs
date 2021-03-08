use structopt::StructOpt;
use std::str::FromStr;
use dialoguer::{
    Input,
};
use async_recursion::async_recursion;

use super::super::receiver::{
    ActionSubcommand,
    CliActionSkipSubcommand
};


#[derive(Debug)]
pub struct DeleteAccessKeyAction {
    pub public_key: String,
    pub next_action: Box<ActionSubcommand>
}

#[derive(Debug, StructOpt)]
pub struct CliDeleteAccessKeyAction {
    #[structopt(long)]
    public_key: Option<String>,
    #[structopt(subcommand)]
    next_action: Option<CliActionSkipSubcommand>
}

impl From<CliDeleteAccessKeyAction> for DeleteAccessKeyAction {
    fn from(item: CliDeleteAccessKeyAction) -> Self {
        let public_key: String = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyAction::input_public_key()
        };
        let next_action: Box<ActionSubcommand> = match item.next_action {
            Some(cli_skip_action) => {
                Box::new(ActionSubcommand::from(cli_skip_action))
            },
            None => Box::new(ActionSubcommand::choose_action_command()) 
        };
        DeleteAccessKeyAction {
            public_key,
            next_action
        }
    }
}

impl DeleteAccessKeyAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        // public_key_string: String,
    ) {
        println!("DeleteAccessKeyAction process: self:\n       {:?}", &self);
        println!("DeleteAccessKeyAction process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let public_key = near_crypto::PublicKey::from_str(&self.public_key).unwrap();
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key
            }
        );
        let mut actions= prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            .. prepopulated_unsigned_transaction
        };
        match *self.next_action {
            ActionSubcommand::TransferNEARTokens(args_transfer) => args_transfer.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::CallFunction(args_function) => {},
            // ActionSubcommand::StakeNEARTokens(args_stake) => {},
            ActionSubcommand::CreateAccount(args_create_account) => args_create_account.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::DeleteAccount(args_delete_account) => args_delete_account.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::AddAccessKey(args_add_public_key) => args_add_public_key.process(unsigned_transaction, selected_server_url, "".to_string()).await,
            ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::Skip(args_skip) => args_skip.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }

    }
    pub fn input_public_key() -> String {
        Input::new()
            .with_prompt("Enter the access key to remove it")
            .interact_text()
            .unwrap()
    }
}
