use color_eyre::eyre::Context;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CallFunctionViewContext)]
pub struct CallFunctionView {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subargs)]
    /// Select function
    function: Function,
}

#[derive(Clone)]
pub struct CallFunctionViewContext {
    global_context: crate::GlobalContext,
    contract_account_id: near_primitives::types::AccountId,
}

impl CallFunctionViewContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<CallFunctionView as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            contract_account_id: scope.contract_account_id.clone().into(),
        })
    }
}

impl CallFunctionView {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = CallFunctionViewContext)]
#[interactive_clap(output_context = FunctionContext)]
pub struct Function {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct FunctionContext(crate::network_view_at_block::ArgsForViewContext);

impl FunctionContext {
    pub fn from_previous_context(
        previous_context: CallFunctionViewContext,
        scope: &<Function as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let function_args = scope.function_args.clone();
            let function_args_type = scope.function_args_type.clone();
            let account_id: near_primitives::types::AccountId = previous_context.contract_account_id.clone();
            let function_name = scope.function_name.clone();

            move |network_config, block_reference| {
                call_view_function(
                    previous_context.global_context.teach_me,
                    network_config,
                    &account_id,
                    &function_name,
                    function_args.clone(),
                    function_args_type.clone(),
                    block_reference
                )
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.contract_account_id],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<FunctionContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: FunctionContext) -> Self {
        item.0
    }
}

impl Function {
    fn input_function_args_type(
        _context: &CallFunctionViewContext,
    ) -> color_eyre::eyre::Result<Option<super::call_function_args_type::FunctionArgsType>> {
        super::call_function_args_type::input_function_args_type()
    }

    fn input_function_name(
        context: &CallFunctionViewContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        super::input_view_function_name(&context.global_context, &context.contract_account_id)
    }
}

#[tracing::instrument(name = "Getting a response to a view method ...", skip_all)]
fn call_view_function(
    teach_me: bool,
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    function_name: &str,
    function_args: String,
    function_args_type: super::call_function_args_type::FunctionArgsType,
    block_reference: &near_primitives::types::BlockReference,
) -> crate::CliResult {
    let args = super::call_function_args_type::function_args(function_args, function_args_type)?;
    let call_result = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            teach_me,
            account_id,
            function_name,
            args,
            block_reference.clone(),
        )
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query for view method: '{}' (contract <{}> on network <{}>)",
                function_name, account_id, network_config.network_name
            )
        })?;
    call_result.print_logs();
    eprintln!("Result:");
    if call_result.result.is_empty() {
        eprintln!("Empty result");
    } else if let Ok(json_result) = call_result.parse_result_from_json::<serde_json::Value>() {
        println!("{}", serde_json::to_string_pretty(&json_result)?);
    } else if let Ok(string_result) = String::from_utf8(call_result.result) {
        println!("{string_result}");
    } else {
        eprintln!("The returned value is not printable (binary data)");
    }
    eprintln!("--------------");
    Ok(())
}
