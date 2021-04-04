use structopt::StructOpt;
use dialoguer::{theme::ColorfulTheme, Input, Select};
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
    #[structopt(long)]
    method_name: Option<String>,
    #[structopt(long)]
    args: Option<String>,
    #[structopt(long)]
    gas: Option<near_primitives::types::Gas>,
    #[structopt(long)]
    deposit: Option<crate::common::NearBalance>,
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
                cli_args.into_bytes()
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
        let choose_mode = vec![
            "Yes, I want to enter a different value",
            "No, I see no problem with default value",
        ];
        println!();
        let select_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "The default is 1000000000 (1 gigagas). Do you want to change this value?"
            )
            .items(&choose_mode)
            .default(0)
            .interact()
            .unwrap();
        match choose_mode[select_mode] {
            "Yes, I want to enter a different value" => {
                println!();
                let gas: u64 = loop {
                    let input_gas: crate::common::NearGas = Input::new()
                        .with_prompt("Enter a gas for function.")
                        .interact_text()
                        .unwrap();
                    let gas: u64 = match input_gas {
                        crate::common::NearGas(num) => num
                    };
                    if gas <= 200000000000000 {
                        break gas;
                    } else {
                        println!("You need to enter a value of no more than 200 TERAGAS")
                    }
                };
                gas
            }
            "No, I see no problem with default value" => {
                1000000000
            }
            _ => unreachable!("Error"),
        }
    }
    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function.")
            .interact_text()
            .unwrap();
        input.into_bytes()
    }
    fn input_deposit() -> near_primitives::types::Balance {
        println!();
        let choose_mode = vec![
            "Yes, I want to enter a different value",
            "No, I see no problem with default value",
        ];
        println!();
        let select_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "The default is 0. Do you want to change this value?"
            )
            .items(&choose_mode)
            .default(0)
            .interact()
            .unwrap();
        match choose_mode[select_mode] {
            "Yes, I want to enter a different value" => {
                println!();
                let deposit: crate::common::NearBalance = Input::new()
                    .with_prompt("Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).")
                    .interact_text()
                    .unwrap();
                match deposit {
                    crate::common::NearBalance(num) => num,
                }
            }
            "No, I see no problem with default value" => {
                0
            }
            _ => unreachable!("Error"),
        }
    }
}
