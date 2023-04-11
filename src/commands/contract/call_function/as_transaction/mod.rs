use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CallFunctionPropertiesContext)]
pub struct CallFunctionProperties {
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct CallFunctionPropertiesContext {
    config: crate::config::Config,
    receiver_account_id: near_primitives::types::AccountId,
    function_name: String,
    function_args: Vec<u8>,
}

impl CallFunctionPropertiesContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<CallFunctionProperties as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let function_args = super::call_function_args_type::function_args(
            scope.function_args.clone(),
            scope.function_args_type.clone(),
        )?;
        Ok(Self {
            config: previous_context.0,
            receiver_account_id: scope.contract_account_id.clone().into(),
            function_name: scope.function_name.clone(),
            function_args,
        })
    }
}

impl CallFunctionProperties {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<super::call_function_args_type::FunctionArgsType>> {
        super::call_function_args_type::input_function_args_type()
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = CallFunctionPropertiesContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for a function call
    attached_deposit: Deposit,
}

#[derive(Debug, Clone)]
pub struct PrepaidGasContext {
    config: crate::config::Config,
    receiver_account_id: near_primitives::types::AccountId,
    function_name: String,
    function_args: Vec<u8>,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: CallFunctionPropertiesContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            receiver_account_id: previous_context.receiver_account_id,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: scope.gas.clone(),
        })
    }
}

impl PrepaidGas {
    fn input_gas(
        _context: &CallFunctionPropertiesContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        eprintln!();
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
                        eprintln!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(Some(gas.into()))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: SignerAccountId,
}

#[derive(Debug, Clone)]
pub struct DepositContext {
    config: crate::config::Config,
    receiver_account_id: near_primitives::types::AccountId,
    function_name: String,
    function_args: Vec<u8>,
    gas: crate::common::NearGas,
    deposit: crate::common::NearBalance,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            receiver_account_id: previous_context.receiver_account_id,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: previous_context.gas,
            deposit: scope.deposit.clone(),
        })
    }
}

impl Deposit {
    fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        eprintln!();
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

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DepositContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerAccountIdContext {
    config: crate::config::Config,
    receiver_account_id: near_primitives::types::AccountId,
    function_name: String,
    function_args: Vec<u8>,
    gas: crate::common::NearGas,
    deposit: crate::common::NearBalance,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: DepositContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            receiver_account_id: previous_context.receiver_account_id,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: previous_context.gas,
            deposit: previous_context.deposit,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            receiver_account_id: item.receiver_account_id,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: item.function_name,
                    args: item.function_args,
                    gas: item.gas.inner,
                    deposit: item.deposit.to_yoctonear(),
                },
            )],
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
