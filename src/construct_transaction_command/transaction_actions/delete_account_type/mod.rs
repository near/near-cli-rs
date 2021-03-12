use structopt::StructOpt;
use dialoguer::{
    Input,
};
use async_recursion::async_recursion;

use super::super::receiver::{
    NextAction,
    CliSkipNextAction
};


#[derive(Debug)]
pub struct DeleteAccountAction {
    pub beneficiary_id: String,
    pub next_action: Box<NextAction>
}

#[derive(Debug, StructOpt)]
pub struct CliDeleteAccountAction {
    #[structopt(long)]
    beneficiary_id: Option<String>,
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>
}

impl From<CliDeleteAccountAction> for DeleteAccountAction {
    fn from(item: CliDeleteAccountAction) -> Self {
        let beneficiary_id: String = match item.beneficiary_id {
            Some(cli_account_id) => cli_account_id,
            None => DeleteAccountAction::input_beneficiary_id()
        };
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => {
                Box::new(NextAction::from(cli_skip_action))
            },
            None => Box::new(NextAction::input_next_action()) 
        };
        DeleteAccountAction {
            beneficiary_id,
            next_action
        }
    }
}

impl DeleteAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("DeleteAccountAction process: self:\n       {:?}", &self);
        println!("DeleteAccountAction process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let beneficiary_id: String = self.beneficiary_id.clone();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction {
                beneficiary_id
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
            // ActionSubcommand::AddAccessKey(args_add_access_key) => args_add_access_key.process(unsigned_transaction, selected_server_url, "".to_string()).await,
            // ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::Skip(args_skip) => args_skip.process(unsigned_transaction, selected_server_url).await,
            NextAction::AddAction(select_action) => select_action.process(unsigned_transaction, selected_server_url).await,
            NextAction::Skip(skip_action) => skip_action.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
    pub fn input_beneficiary_id() -> String {
        println!();
        Input::new()
            .with_prompt("Enter the beneficiary ID to delete this account ID")
            .interact_text()
            .unwrap()
        }
}
