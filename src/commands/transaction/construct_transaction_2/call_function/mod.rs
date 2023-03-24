use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = FunctionCallActionContext)]
pub struct FunctionCallAction {
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type:
        crate::commands::contract::call_function::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct FunctionCallActionContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    actions: Vec<near_primitives::transaction::Action>,
    function_name: String,
    function_args: Vec<u8>,
}

impl FunctionCallActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        scope: &<FunctionCallAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let function_args =
            crate::commands::contract::call_function::call_function_args_type::function_args(
                scope.function_args.clone(),
                scope.function_args_type.clone(),
            )?;
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: previous_context.actions,
            function_name: scope.function_name.clone(),
            function_args,
        })
    }
}

impl FunctionCallAction {
    fn input_function_args_type(
        _context: &super::super::ConstructTransactionActionContext,
    ) -> color_eyre::eyre::Result<
        Option<crate::commands::contract::call_function::call_function_args_type::FunctionArgsType>,
    > {
        crate::commands::contract::call_function::call_function_args_type::input_function_args_type(
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = FunctionCallActionContext)]
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
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    actions: Vec<near_primitives::transaction::Action>,
    function_name: String,
    function_args: Vec<u8>,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: FunctionCallActionContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: previous_context.actions,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: scope.gas.clone(),
        })
    }
}

impl PrepaidGas {
    fn input_gas(
        _context: &FunctionCallActionContext,
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
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    next_action: super::super::construct_transaction_3::NextAction,
}

#[derive(Clone)]
pub struct DepositContext(super::super::ConstructTransactionActionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: previous_context.function_name,
                args: previous_context.function_args,
                gas: previous_context.gas.inner,
                deposit: scope.deposit.clone().to_yoctonear(),
            },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::ConstructTransactionActionContext {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<DepositContext> for super::super::ConstructTransactionActionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}

impl Deposit {
    fn input_deposit(
        _context: &PrepaidGasContext,
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
