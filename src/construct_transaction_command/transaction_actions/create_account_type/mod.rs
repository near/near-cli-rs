use async_recursion::async_recursion;
use structopt::StructOpt;

use super::super::receiver::{CliSkipNextAction, CliNextAction, NextAction};

#[derive(Debug)]
pub struct CreateAccountAction {
    pub next_action: Box<NextAction>,
}

#[derive(Debug, Default, StructOpt)]
pub struct CliCreateAccountAction {
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliCreateAccountAction> for CreateAccountAction {
    fn from(item: CliCreateAccountAction) -> Self {
        let cli_skip_next_action: CliNextAction = match item.next_action {
            Some(cli_skip_action) => CliNextAction::from(cli_skip_action),
            None => NextAction::input_next_action(),
        };
        CreateAccountAction { next_action: Box::new(NextAction::from(cli_skip_next_action)) }
    }
}

impl CreateAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        println!("CreateAccountAction process: self:\n       {:?}", &self);
        println!(
            "CreateAccountAction process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
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
