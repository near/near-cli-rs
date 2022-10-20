use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionAction {
    ///What is the name of the function?
    function_name: String,
    ///Enter arguments to this function
    function_args: String,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_default_input_arg)]
    ///Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_default_input_arg)]
    ///Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl CallFunctionAction {
    fn input_gas(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearGas> {
        println!();
        let gas: u64 = loop {
            let input_gas: crate::common::NearGas = Input::new()
                .with_prompt("Enter gas for function call")
                .with_initial_text("100 TeraGas")
                .interact_text()?;
            let crate::common::NearGas { inner: num } = input_gas;
            let gas = num;
            if gas <= 300000000000000 {
                break gas;
            } else {
                println!("You need to enter a value of no more than 300 TERAGAS")
            }
        };
        Ok(gas.into())
    }

    fn input_deposit(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()?;
        Ok(deposit)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.function_name.clone(),
                args: self.function_args.clone().into_bytes(),
                gas: self.gas.clone().inner,
                deposit: self.deposit.clone().to_yoctonear(),
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config,
        )
        .await
    }
}
