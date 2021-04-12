use dialoguer::Input;


/// вызов CallFunction
#[derive(Debug, Default, clap::Clap)]
pub struct CliCallFunctionView {
    method_name: Option<String>,
    args: Option<String>,
}

#[derive(Debug)]
pub struct CallFunctionView {
    method_name: String,
    args: Vec<u8>,
}

impl From<CliCallFunctionView> for CallFunctionView {
    fn from(item: CliCallFunctionView) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionView::input_method_name()
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionView::input_args()
        };
        Self {
            method_name,
            args,
        }
    }
}

impl CallFunctionView {
    fn input_method_name() -> String {
        println!();
        Input::new()
            .with_prompt("Enter a method name")
            .interact_text()
            .unwrap()
    }

    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap();
        input.into_bytes()
    }

    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        selected_server_url: url::Url,
        contract_account_id: String,
    ) -> crate::CliResult {
        let method_name = self.method_name.clone();
        let args: near_primitives::types::FunctionArgs = near_primitives::types::FunctionArgs::from(self.args.clone());
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: contract_account_id,
                    method_name,
                    args,
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view method: {:?}",
                    err
                ))
            })?;
        let call_result = if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(
            result,
        ) = query_view_method_response.kind
        {
            result.result
        } else {
            return Err(color_eyre::Report::msg(format!(
                "Error current_nonce"
            )));
        };
        let call_result_str = String::from_utf8(call_result).unwrap();
        let serde_call_result: serde_json::Value = serde_json::from_str(&call_result_str)
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "serde json: {:?}",
                    err
                ))
            })?;
        println!("--------------");
        println!();
        println!("{}", serde_json::to_string_pretty(&serde_call_result).unwrap() );
        Ok(())
    }
}
