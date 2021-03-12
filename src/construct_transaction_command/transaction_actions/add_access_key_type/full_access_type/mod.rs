use structopt::StructOpt;
use std::str::FromStr;
use async_recursion::async_recursion;

use crate::construct_transaction_command::receiver::{
    NextAction,
    CliSkipNextAction
};


#[derive(Debug)]
pub struct FullAccessType {
    pub next_action: Box<NextAction>
}

#[derive(Debug, StructOpt)]
pub struct CliFullAccessType {
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>
}

impl From<CliFullAccessType> for FullAccessType {
    fn from(item: CliFullAccessType) -> Self {
        
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => {
                Box::new(NextAction::from(cli_skip_action))
            },
            None => Box::new(NextAction::input_next_action()) 
        };
        FullAccessType {
            next_action,
        }
    }
}

impl FullAccessType {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        public_key_string: String,
    ) {
        println!("FullAccessType process: self:\n       {:?}", &self);
        println!("FullAccessType process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let public_key = near_crypto::PublicKey::from_str(&public_key_string).unwrap();
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
                nonce,
                permission: near_primitives::account::AccessKeyPermission::FullAccess
            };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key
            }
        );
        let mut actions= prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            .. prepopulated_unsigned_transaction
        };
        match *self.next_action {
            // ActionSubcommand::TransferNEARTokens(args_transfer) => args_transfer.process(unsigned_transaction, selected_server_url).await,
            // // ActionSubcommand::CallFunction(args_function) => {},
            // // ActionSubcommand::StakeNEARTokens(args_stake) => {},
            // ActionSubcommand::CreateAccount(args_create_account) => args_create_account.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::DeleteAccount(args_delete_account) => args_delete_account.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::AddAccessKey(args_add_access_key) => args_add_access_key.process(unsigned_transaction, selected_server_url, public_key_string).await,
            // ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::Skip(args_skip) => args_skip.process(unsigned_transaction, selected_server_url).await,
            NextAction::AddAction(select_action) => select_action.process(unsigned_transaction, selected_server_url).await,
            NextAction::Skip(skip_action) => skip_action.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
}
