use std::{str::FromStr, vec};
use structopt::StructOpt;

use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

use crate::construct_transaction_command::receiver::{CliSkipNextAction, CliNextAction, NextAction};

#[derive(Debug)]
pub struct FunctionCallType {
    pub allowance: Option<near_primitives::types::Balance>,
    pub receiver_id: near_primitives::types::AccountId,
    pub method_names: Vec<String>,
    pub next_action: Box<NextAction>,
}

#[derive(Debug, Default, StructOpt)]
pub struct CliFunctionCallType {
    #[structopt(long)]
    allowance: Option<crate::common::NearBalance>,
    #[structopt(long)]
    receiver_id: Option<near_primitives::types::AccountId>,
    #[structopt(long)]
    method_names: Option<String>,
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

impl From<CliFunctionCallType> for FunctionCallType {
    fn from(item: CliFunctionCallType) -> Self {
        let allowance: Option<near_primitives::types::Balance> = match item.allowance {
            Some(cli_allowance) => {
                let allowance = match cli_allowance {
                    crate::common::NearBalance(num) => num,
                };
                Some(allowance)
            }
            None => FunctionCallType::input_allowance(),
        };
        let receiver_id: near_primitives::types::AccountId = match item.receiver_id {
            Some(cli_receiver_id) => near_primitives::types::AccountId::from(cli_receiver_id),
            None => FunctionCallType::input_receiver_id(),
        };
        let method_names: Vec<String> = match item.method_names {
            Some(cli_method_names) => {
                if cli_method_names.is_empty() {
                    vec![]
                } else {
                    cli_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>()
                }
            }
            None => FunctionCallType::input_method_names(),
        };
        let cli_skip_next_action: CliNextAction = match item.next_action {
            Some(cli_skip_action) => CliNextAction::from(cli_skip_action),
            None => NextAction::input_next_action(),
        };
        FunctionCallType {
            allowance,
            receiver_id,
            method_names,
            next_action: Box::new(NextAction::from(cli_skip_next_action)),
        }
    }
}

impl FunctionCallType {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        println!("FunctionCallType process: self:\n       {:?}", &self);
        println!(
            "FunctionCallType process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: self.allowance.clone(),
                    receiver_id: self.receiver_id.clone(),
                    method_names: self.method_names.clone(),
                },
            ),
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
    pub fn input_method_names() -> Vec<String> {
        println!();
        let choose_input = vec![
            "Yes, I want to input a list of method names that can be used",
            "No, I don't to input a list of method names that can be used",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input a list of method names that can be used")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let mut input_method_names: String = Input::new()
                    .with_prompt("Enter a list of method names that can be used. The access key only allows transactions with the function call of one of the given method names. Empty list means any method name can be used.")
                    .interact_text()
                    .unwrap();
                if input_method_names.contains("\"") {
                    input_method_names.clear()
                };
                if input_method_names.is_empty() {
                    vec![]
                } else {
                    input_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>()
                }
            }
            Some(1) => vec![],
            _ => unreachable!("Error"),
        }
    }
    pub fn input_allowance() -> Option<near_primitives::types::Balance> {
        println!();
        let choose_input = vec![
            "Yes, I want to input allowance for receiver ID",
            "No, I don't to input allowance for receiver ID",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input an allowance for receiver ID")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let input: String = Input::new()
                    .with_prompt("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees.")
                    .interact_text()
                    .unwrap();
                let allowance_near_balance: crate::common::NearBalance = crate::common::NearBalance::from_str(&input).unwrap();
                let allowance = match allowance_near_balance {
                    crate::common::NearBalance(num) => num,
                };
                Some(allowance)
            }
            Some(1) => None,
            _ => unreachable!("Error"),
        }
    }
    pub fn input_receiver_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("Enter a receiver to use by this access key to pay for function call gas and transaction fees.")
            .interact_text()
            .unwrap()
    }
}
