use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CallFunctionViewContext)]
pub struct CallFunctionView {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the account ID?
    account_id: crate::types::account_id::AccountId,
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct CallFunctionViewContext(crate::network_view_at_block::ArgsForViewContext);

impl CallFunctionViewContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<CallFunctionView as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let function_args = scope.function_args.clone();
            let function_args_type = scope.function_args_type.clone();
            let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();
            let function_name = scope.function_name.clone();

            move |network_config, block_reference| {
                let args = super::call_function_args_type::function_args(
                    function_args.clone(),
                    function_args_type.clone(),
                )?;

                let call_result = network_config
                .json_rpc_client()
                .blocking_call_view_function(
                    &account_id,
                    &function_name,
                    args,
                    block_reference.clone(),
                )?;
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
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<CallFunctionViewContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: CallFunctionViewContext) -> Self {
        item.0
    }
}

impl CallFunctionView {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<super::call_function_args_type::FunctionArgsType>> {
        super::call_function_args_type::input_function_args_type()
    }
}

impl CallFunctionView {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the account ID?",
        )
    }
}
