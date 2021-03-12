use structopt::StructOpt;
use async_recursion::async_recursion;

use super::super::receiver::{
    // ActionSubcommand,
    CliSkipNextAction,
    NextAction,
    // CliNextAction
    
};

#[derive(Debug)]
pub struct CreateAccountAction {
    pub next_action: Box<NextAction>
}

#[derive(Debug, StructOpt)]
pub struct CliCreateAccountAction {
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>
}

impl From<CliCreateAccountAction> for CreateAccountAction {
    fn from(item: CliCreateAccountAction) -> Self {
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_next_action) => {
                Box::new(NextAction::from(cli_next_action))
            },
            None => Box::new(NextAction::input_next_action()) 
        };
        CreateAccountAction {
            next_action
        }
    }
}

impl CreateAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        // public_key_string: String,
    ) {
        println!("CreateAccountAction process: self:\n       {:?}", &self);
        println!("CreateAccountAction process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {}
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
            // ActionSubcommand::AddAccessKey(args_add_access_key) => args_add_access_key.process(unsigned_transaction, selected_server_url, "".to_string()).await,
            // ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::Skip(args_skip) => args_skip.process(unsigned_transaction, selected_server_url).await,
            NextAction::AddAction(select_action) => select_action.process(unsigned_transaction, selected_server_url).await,
            NextAction::Skip(skip_action) => skip_action.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
}
