use structopt::StructOpt;
use dialoguer::{
    Input,
};
use std::num::ParseIntError;
use std::str::FromStr;
use async_recursion::async_recursion;

use super::super::receiver::{
    NextAction,
    CliSkipNextAction
};


#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: NearBalance,
    pub next_action: Box<NextAction>
}

impl TransferNEARTokensAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("TransferNEARTokens process: self:\n       {:?}", &self);
        println!("TransferNEARTokens process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let amount = match self.amount {
            NearBalance(num) => num
        };
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: amount,
            },
        );
        let mut actions= prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            .. prepopulated_unsigned_transaction
        };
        match *self.next_action {
            NextAction::AddAction(select_action) => select_action.process(unsigned_transaction, selected_server_url).await,
            NextAction::Skip(skip_action) => skip_action.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct CliTransferNEARTokensAction {
    amount: Option<NearBalance>,
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction> 
}

impl NearBalance {
    pub fn input_amount() -> Self {
        let input: String = Input::new()
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR)")
            .interact_text()
            .unwrap();
        NearBalance::from_str(&input).unwrap()
    }
}

#[derive(Debug)]
pub struct NearBalance (u128);

impl FromStr for NearBalance {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u128 = s.parse().unwrap_or_else(|ParseIntError| -> u128 {
            let mut s: String = s.to_string().clone();
            s.make_ascii_uppercase();
            match s.contains("NEAR") {
                true => {
                    let num:u128 = s.trim_matches(char::is_alphabetic)
                        .parse()
                        .unwrap();
                    num * 10u128.pow(24)
                },
                _ => 0
            }
        });
        Ok(NearBalance(number))
    }
}

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => NearBalance::input_amount()
        };
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => {
                Box::new(NextAction::from(cli_skip_action))
            },
            None => Box::new(NextAction::input_next_action()) 
        };
        TransferNEARTokensAction {
            amount,
            next_action
        }
    }
}

