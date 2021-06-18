use dialoguer::Input;

/// вызов CallFunction
#[derive(Debug, Default, clap::Clap)]
pub struct CliCallFunctionAction {
    method_name: Option<String>,
    args: Option<String>,
    #[clap(long = "attached-deposit")]
    deposit: Option<crate::common::NearBalance>,
    #[clap(long = "prepaid-gas")]
    gas: Option<crate::common::NearGas>,
    #[clap(subcommand)]
    pub sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct CallFunctionAction {
    method_name: String,
    args: Vec<u8>,
    gas: near_primitives::types::Gas,
    deposit: near_primitives::types::Balance,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliCallFunctionAction> for CallFunctionAction {
    fn from(item: CliCallFunctionAction) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionAction::input_method_name(),
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionAction::input_args(),
        };
        let gas: near_primitives::types::Gas = match item.gas {
            Some(cli_gas) => match cli_gas {
                crate::common::NearGas { inner: num } => num,
            },
            None => CallFunctionAction::input_gas(),
        };
        let deposit: near_primitives::types::Balance = match item.deposit {
            Some(cli_deposit) => cli_deposit.to_yoctonear(),
            None => CallFunctionAction::input_deposit(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            method_name,
            args,
            gas,
            deposit,
            sign_option,
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
                crate::common::NearGas { inner: num } => num,
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
            .with_prompt(
                "Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()
            .unwrap();
        deposit.to_yoctonear()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        file_path: std::path::PathBuf,
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
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config)
            .await?
        {
            Some(transaction_info) => {
                match transaction_info.status {
                    near_primitives::views::FinalExecutionStatus::NotStarted => {
                        println!("NotStarted")
                    }
                    near_primitives::views::FinalExecutionStatus::Started => println!("Started"),
                    near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                        crate::common::print_transaction_error(tx_execution_error).await
                    }
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                        match transaction_info.transaction.actions[0] {
                            near_primitives::views::ActionView::DeployContract { code: _ } => {
                                println!(
                                    "\n Contract code {:?} has been successfully deployed.",
                                    file_path
                                );
                            }
                            _ => unreachable!("Error")
                        }
                    }
                }
                println!("\nTransaction Id {id}.\n\nTo see the transaction in the transaction explorer, please open this url in your browser:
                        \nhttps://explorer.testnet.near.org/transactions/{id}\n", id=transaction_info.transaction_outcome.id);
            }
            None => {}
        };
        Ok(())
    }
}
