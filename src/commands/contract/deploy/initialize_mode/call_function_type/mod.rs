use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ContractFileContext)]
#[interactive_clap(output_context = CallFunctionActionContext)]
pub struct CallFunctionAction {
    /// What is the name of the function?
    function_name: String,
    /// Enter arguments to this function
    function_args: String,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct CallFunctionActionContext {
    config: crate::config::Config,
    receiver_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
    code: Vec<u8>,
    function_name: String,
    function_args: String,
    gas: crate::common::NearGas,
    deposit: crate::common::NearBalance,
}

impl CallFunctionActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ContractFileContext,
        scope: &<CallFunctionAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            code: previous_context.code,
            function_name: scope.function_name.clone(),
            function_args: scope.function_args.clone(),
            gas: scope.gas.clone(),
            deposit: scope.deposit.clone(),
        })
    }
}

impl From<CallFunctionActionContext> for crate::commands::ActionContext {
    fn from(item: CallFunctionActionContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            receiver_account_id: item.receiver_account_id,
            actions: vec![
                near_primitives::transaction::Action::DeployContract(
                    near_primitives::transaction::DeployContractAction { code: item.code },
                ),
                near_primitives::transaction::Action::FunctionCall(
                    near_primitives::transaction::FunctionCallAction {
                        method_name: item.function_name,
                        args: item.function_args.into_bytes(),
                        gas: item.gas.inner,
                        deposit: item.deposit.to_yoctonear(),
                    },
                ),
            ],
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_after_getting_network_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl CallFunctionAction {
    fn input_gas(
        _context: &super::super::ContractFileContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
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
        Ok(Some(gas.into()))
    }

    fn input_deposit(
        _context: &super::super::ContractFileContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        println!();
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("0 NEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}
