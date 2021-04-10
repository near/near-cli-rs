use async_recursion::async_recursion;
use dialoguer::Input;


/// вызов CallFunction
#[derive(Debug, Default, clap::Clap)]
pub struct CliCallFunctionAction {
    #[clap(long)]
    method_name: Option<String>,
    #[clap(long)]
    args: Option<String>,
    #[clap(long)]
    gas: Option<crate::common::NearGas>,
    #[clap(long)]
    deposit: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>
}

#[derive(Debug)]
pub struct CallFunctionAction {
    method_name: String,
    args: Vec<u8>,
    gas: near_primitives::types::Gas,
    deposit: near_primitives::types::Balance,
    next_action: Box<super::NextAction>
}

impl From<CliCallFunctionAction> for CallFunctionAction {
    fn from(item: CliCallFunctionAction) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionAction::input_method_name()
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionAction::input_args()
        };
        let gas: near_primitives::types::Gas = match item.gas {
            Some(cli_gas) => {
                match cli_gas {
                    crate::common::NearGas {inner: num} => num
                }
            },
            None => CallFunctionAction::input_gas()
        };
        let deposit: near_primitives::types::Balance = match item.deposit {
            Some(cli_deposit) => {
                match cli_deposit {
                    crate::common::NearBalance {inner: num} => num
                }
            },
            None => CallFunctionAction::input_deposit()
        };
        let skip_next_action: super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::NextAction::from(cli_skip_action),
            None => super::NextAction::input_next_action(),
        };
        Self {
            method_name,
            args,
            gas,
            deposit,
            next_action: Box::new(skip_next_action),
        }
    }
}

impl CallFunctionAction {
    fn input_method_name() -> String {
        println!();
        Input::new()
            .with_prompt("Enter a method name")
            .interact_text()
            .unwrap()
    }

    fn input_gas() -> near_primitives::types::Gas {
        println!();
        let gas: u64 = loop {
            let input_gas: crate::common::NearGas = Input::new()
                .with_prompt("Enter a gas for function")
                .with_initial_text("100 TeraGas")
                .interact_text()
                .unwrap();
            let gas: u64 = match input_gas {
                crate::common::NearGas { inner: num } => num
            };
            if gas <= 200000000000000 {
                break gas;
            } else {
                println!("You need to enter a value of no more than 200 TERAGAS")
            }
        };
        gas
    }
    
    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap();
        input.into_bytes()
    }

    fn input_deposit() -> near_primitives::types::Balance {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt("Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).")
            .with_initial_text("0 Near")
            .interact_text()
            .unwrap();
        match deposit {
            crate::common::NearBalance {inner: num} => num,
        }
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
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
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}
