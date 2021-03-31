use structopt::StructOpt;
use dialoguer::Input;
use async_recursion::async_recursion;

use super::super::receiver::{CliSkipNextAction, CliNextAction, NextAction};

#[derive(Debug)]
pub struct CallFunctionAction {
    method_name: String,
    args: Vec<u8>,
    gas: near_primitives::types::Gas,  // default 1000000000
    deposit: near_primitives::types::Balance,  // default 0
    next_action: Box<NextAction>
}

#[derive(Debug, Default, StructOpt)]
pub struct CliCallFunctionAction {
    method_name: Option<String>,
    args: Option<String>,
    gas: Option<near_primitives::types::Gas>,  // default 1000000000
    deposit: Option<crate::common::NearBalance>,  // default 0
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>
}

impl From<CliCallFunctionAction> for CallFunctionAction {
    fn from(item: CliCallFunctionAction) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionAction::input_method_name()
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => {
                println!("CallFunctionAction args: {:?}", &cli_args);
                [1,2,3].to_vec()
            },
            None => CallFunctionAction::input_args()
        };
        let gas: near_primitives::types::Gas = match item.gas {
            Some(cli_gas) => cli_gas,
            None => CallFunctionAction::input_gas()
        };
        let deposit: near_primitives::types::Balance = match item.deposit {
            Some(cli_deposit) => {
                match cli_deposit {
                    crate::common::NearBalance(num) => num
                }
            },
            None => CallFunctionAction::input_deposit()
        };
        let cli_skip_next_action: CliNextAction = match item.next_action {
            Some(cli_skip_action) => CliNextAction::from(cli_skip_action),
            None => NextAction::input_next_action(),
        };
        CallFunctionAction {
            method_name,
            args,
            gas,
            deposit,
            next_action: Box::new(NextAction::from(cli_skip_next_action)),
        }
    }
}

impl CallFunctionAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        println!("CallFunctionAction self: {:?}", &self);
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.method_name.clone(),
                args: self.args.clone(),
                gas: self.gas.clone(),
                deposit: self.deposit.clone(),
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
    fn input_method_name() -> String {
        println!();
        Input::new()
            .with_prompt("Enter a method name.")
            .interact_text()
            .unwrap()
    }
    fn input_gas() -> near_primitives::types::Gas {
        println!();
        Input::new()
            .with_prompt("Enter a gas for function.")
            .interact_text()
            .unwrap()
    }
    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function.")
            .interact_text()
            .unwrap();
        let args = input
            .split(',')
            .map(|x| {
                x.trim()
                .parse::<u8>()
                .map_err(|err| format!("Parsing error : {}", err))
                .unwrap()
            })
            .collect::<Vec<u8>>();
        println!("=======  args: {:?}", &args);
        args
    }
    fn input_deposit() -> near_primitives::types::Balance {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt("Enter a deposit for function.")
            .interact_text()
            .unwrap();
        match deposit {
            crate::common::NearBalance(num) => num,
        }
    }
}
