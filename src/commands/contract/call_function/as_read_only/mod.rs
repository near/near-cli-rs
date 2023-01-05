#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionView {
    ///What is the account ID?
    account_id: crate::types::account_id::AccountId,
    ///What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    ///How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    ///Enter the arguments to this function or the path to the arguments file
    function_args: String,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl CallFunctionView {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<super::call_function_args_type::FunctionArgsType> {
        super::call_function_args_type::input_function_args_type()
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let args = super::call_function_args_type::function_args(
            self.function_args.clone(),
            self.function_args_type.clone(),
        )?;
        let query_view_method_response = self
            .network_config
            .get_network_config(config)
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: self.network_config.get_block_ref(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: self.account_id.clone().into(),
                    method_name: self.function_name.clone(),
                    args: near_primitives::types::FunctionArgs::from(args),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
            })?;
        let call_result =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };

        let serde_call_result = if call_result.result.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&call_result.result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        println!("--------------");
        if call_result.logs.is_empty() {
            println!("No logs")
        } else {
            println!("Logs:");
            println!("  {}", call_result.logs.join("\n  "));
        }
        println!("--------------");
        println!("Result:");
        println!("{}", serde_json::to_string_pretty(&serde_call_result)?);
        println!("--------------");
        Ok(())
    }
}
