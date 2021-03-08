use structopt::StructOpt;
use std::{str::FromStr, vec};
use std::num::ParseIntError;

use dialoguer::{
    Select,
    Input,
    theme::ColorfulTheme,
    console::Term
};
use async_recursion::async_recursion;


use crate::construct_transaction_command::receiver::{
    ActionSubcommand,
    CliActionSkipSubcommand
};


#[derive(Debug)]
pub struct FunctionCallType {
    pub allowance: Option<near_primitives::types::Balance>,
    pub receiver_id: near_primitives::types::AccountId,
    pub method_names: Vec<String>,
    pub next_action: Box<ActionSubcommand>
}

#[derive(Debug, StructOpt)]
pub struct  CliFunctionCallType {
    #[structopt(long)]
    allowance: Option<NearBalance>,
    #[structopt(long)]
    receiver_id: Option<near_primitives::types::AccountId>,
    #[structopt(long)]
    method_names: Option<String>,
    #[structopt(subcommand)]
    next_action: Option<CliActionSkipSubcommand>
}

impl From<CliFunctionCallType> for FunctionCallType {
    fn from(item: CliFunctionCallType) -> Self {
        let allowance: Option<near_primitives::types::Balance> = match item.allowance {
            Some(cli_allowance) => {
                let allowance = match cli_allowance {
                    NearBalance(num) => num
                };
                Some(allowance)
            },
            None => FunctionCallType::input_allowance()
        }; 
        let receiver_id: near_primitives::types::AccountId = match item.receiver_id {
            Some(cli_receiver_id) => near_primitives::types::AccountId::from(cli_receiver_id),
            None => FunctionCallType::input_receiver_id()
        }; 
        let method_names: Vec<String> = match item.method_names {
            Some(cli_method_names) => {
                if cli_method_names.is_empty() {
                    vec![]
                } else {
                    cli_method_names.split(',').map(String::from).collect::<Vec<String>>()
                }
            },
            None => FunctionCallType::input_method_names()
        }; 
        let next_action: Box<ActionSubcommand> = match item.next_action {
            Some(cli_skip_action) => {
                Box::new(ActionSubcommand::from(cli_skip_action))
            },
            None => Box::new(ActionSubcommand::choose_action_command()) 
        };
        FunctionCallType {
            allowance,
            receiver_id,
            method_names,
            next_action
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
        public_key_string: String,
    ) {
        println!("FunctionCallType process: self:\n       {:?}", &self);
        println!("FunctionCallType process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        let public_key = near_crypto::PublicKey::from_str(&public_key_string).unwrap();
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
                nonce,
                permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                    near_primitives::account::FunctionCallPermission {
                        allowance: self.allowance.clone(),
                        receiver_id: self.receiver_id.clone(),
                        method_names: self.method_names.clone()
                    }
                )
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
            ActionSubcommand::TransferNEARTokens(args_transfer) => args_transfer.process(unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::CallFunction(args_function) => {},
            // ActionSubcommand::StakeNEARTokens(args_stake) => {},
            ActionSubcommand::CreateAccount(args_create_account) => args_create_account.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::DeleteAccount(args_delete_account) => args_delete_account.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::AddAccessKey(args_add_access_key) => args_add_access_key.process(unsigned_transaction, selected_server_url, public_key_string).await,
            ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(unsigned_transaction, selected_server_url).await,
            ActionSubcommand::Skip(args_skip) => args_skip.process(unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
    pub fn input_method_names() -> Vec<String> {
        println!();
        let choose_input = vec![
            "Yes, I want to input a list of method names that can be used",
            "No, I don't to input a list of method names that can be used"
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
                if input_method_names.contains("\"") {input_method_names.clear()};
                if input_method_names.is_empty() {
                    vec![]
                } else {
                    input_method_names.split(',').map(String::from).collect::<Vec<String>>()
                }
            },
            Some(1) => vec![],
            _ => unreachable!("Error")
        }
    }
    pub fn input_allowance() -> Option<near_primitives::types::Balance> {
        println!();
        let choose_input = vec![
            "Yes, I want to input allowance for receiver ID",
            "No, I don't to input allowance for receiver ID"
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
                let allowance_near_balance: NearBalance = NearBalance::from_str(&input).unwrap();
                let allowance = match allowance_near_balance {
                    NearBalance(num) => num
                };
                Some(allowance)
            },
            Some(1) => None,
            _ => unreachable!("Error")
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
