use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = FunctionCallActionContext)]
pub struct FunctionCallAction {
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type:
        crate::commands::contract::call_function::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct FunctionCallActionContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    actions: Vec<near_primitives::transaction::Action>,
    function_name: String,
    function_args: Vec<u8>,
}

impl FunctionCallActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<FunctionCallAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let function_args =
            crate::commands::contract::call_function::call_function_args_type::function_args(
                scope.function_args.clone(),
                scope.function_args_type.clone(),
            )?;
        Ok(Self {
            global_context: previous_context.global_context,
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
        _context: &super::super::super::ConstructTransactionContext,
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
    /// Enter gas for function call:
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for a function call
    attached_deposit: Deposit,
}

#[derive(Debug, Clone)]
pub struct PrepaidGasContext {
    global_context: crate::GlobalContext,
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
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: previous_context.actions,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    fn input_gas(
        _context: &FunctionCallActionContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        eprintln!();
        let gas = loop {
            match crate::common::NearGas::from_str(
                &Text::new("Enter gas for function call:")
                    .with_initial_value("100 TeraGas")
                    .prompt()?,
            ) {
                Ok(input_gas) => {
                    if input_gas <= near_gas::NearGas::from_tgas(300) {
                        break input_gas;
                    } else {
                        eprintln!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(Some(gas))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call:
    deposit: crate::types::near_token::NearToken,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_2::NextAction,
}

#[derive(Debug, Clone)]
pub struct DepositContext(super::super::super::ConstructTransactionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::FunctionCall(Box::new(
            near_primitives::transaction::FunctionCallAction {
                method_name: previous_context.function_name,
                args: previous_context.function_args,
                gas: previous_context.gas.as_gas(),
                deposit: scope.deposit.clone().as_yoctonear(),
            },
        ));
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<DepositContext> for super::super::super::ConstructTransactionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}

impl Deposit {
    fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        eprintln!();
        match crate::types::near_token::NearToken::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear):",
            )
            .with_initial_value("0 NEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}
