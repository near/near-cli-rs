use async_recursion::async_recursion;
use inquire::Text;
use std::str::FromStr;

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
    #[interactive_clap(subcommand)]
    next_action: super::BoxNextAction,
}

impl CallFunctionAction {
    fn input_gas(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearGas> {
        println!();
        let gas: u64 = loop {
            match crate::common::NearGas::from_str(
                &Text::new("Enter gas for function call")
                    .with_initial_value("100 TeraGas")
                    .prompt()?,
            ) {
                Ok(input_gas) => {
                    let crate::common::NearGas { inner: num } = input_gas;
                    let gas = num;
                    if gas <= 300000000000000 {
                        break gas;
                    } else {
                        println!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(gas.into())
    }

    fn input_deposit(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("0 NEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(deposit),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }

    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.function_name.clone(),
                args: self.function_args.clone().into_bytes(),
                gas: self.gas.clone().inner,
                deposit: self.deposit.clone().to_yoctonear(),
            },
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action.clone().inner {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
