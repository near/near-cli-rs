use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct CallFunctionAction {
    method_name: String,
    args: String,
    #[interactive_clap(long = "prepaid-gas")]
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    deposit: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CallFunctionAction {
    fn input_method_name(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter a method name")
            .interact_text()
            .unwrap())
    }

    fn input_gas(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<crate::common::NearGas> {
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
            if gas <= 300000000000000 {
                break gas;
            } else {
                println!("You need to enter a value of no more than 200 TERAGAS")
            }
        };
        Ok(gas.into())
    }

    fn input_args(_context: &crate::common::SenderContext) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap())
    }

    fn input_deposit(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()
            .unwrap();
        Ok(deposit)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.method_name.clone(),
                args: self.args.clone().into_bytes(),
                gas: self.gas.clone().inner,
                deposit: self.deposit.clone().to_yoctonear(),
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
            .process(unsigned_transaction, network_connection_config.clone())
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                );
            }
            None => {}
        };
        Ok(())
    }
}
