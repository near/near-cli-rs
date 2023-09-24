use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ContractFileContext)]
#[interactive_clap(output_context = CallFunctionActionContext)]
pub struct CallFunctionAction {
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type:
        super::super::super::call_function::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct CallFunctionActionContext {
    global_context: crate::GlobalContext,
    receiver_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
    code: Vec<u8>,
    function_name: String,
    function_args: Vec<u8>,
}

impl CallFunctionActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ContractFileContext,
        scope: &<CallFunctionAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let function_args =
            super::super::super::call_function::call_function_args_type::function_args(
                scope.function_args.clone(),
                scope.function_args_type.clone(),
            )?;
        Ok(Self {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            code: previous_context.code,
            function_name: scope.function_name.clone(),
            function_args,
        })
    }
}

impl CallFunctionAction {
    fn input_function_args_type(
        _context: &super::super::ContractFileContext,
    ) -> color_eyre::eyre::Result<
        Option<super::super::super::call_function::call_function_args_type::FunctionArgsType>,
    > {
        super::super::super::call_function::call_function_args_type::input_function_args_type()
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = CallFunctionActionContext)]
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
    receiver_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
    code: Vec<u8>,
    function_name: String,
    function_args: Vec<u8>,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: CallFunctionActionContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            code: previous_context.code,
            function_name: previous_context.function_name,
            function_args: previous_context.function_args,
            gas: scope.gas.clone(),
        })
    }
}

impl PrepaidGas {
    fn input_gas(
        _context: &CallFunctionActionContext,
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
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct DepositContext(crate::commands::ActionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let deposit = scope.deposit.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = previous_context.signer_account_id.clone();
                let receiver_account_id = previous_context.receiver_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: receiver_account_id.clone(),
                        actions: vec![
                            near_primitives::transaction::Action::DeployContract(
                                near_primitives::transaction::DeployContractAction {
                                    code: previous_context.code.clone(),
                                },
                            ),
                            near_primitives::transaction::Action::FunctionCall(
                                near_primitives::transaction::FunctionCallAction {
                                    method_name: previous_context.function_name.clone(),
                                    args: previous_context.function_args.clone(),
                                    gas: previous_context.gas.as_gas(),
                                    deposit: deposit.to_yoctonear(),
                                },
                            ),
                        ],
                    })
                }
            });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![
                previous_context.signer_account_id.clone(),
                previous_context.receiver_account_id.clone(),
            ],
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }))
    }
}

impl From<DepositContext> for crate::commands::ActionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}

impl Deposit {
    fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        eprintln!();
        match crate::common::NearBalance::from_str(
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
