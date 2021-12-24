use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl BlockIdHeight {
    pub fn input_block_id_height(
        _context: &super::super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::BlockHeight> {
        Ok(Input::new()
            .with_prompt("Type the block ID height for this contract")
            .interact_text()?)
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        contract_account_id: near_primitives::types::AccountId,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        let args: near_primitives::types::FunctionArgs =
            near_primitives::types::FunctionArgs::from(args);
        let query_view_method_response = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockReference::BlockId(
                    near_primitives::types::BlockId::Height(self.block_id_height.clone()),
                ),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: contract_account_id,
                    method_name,
                    args,
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
                result.result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        let call_result_str = String::from_utf8(call_result)?;
        let serde_call_result: serde_json::Value = serde_json::from_str(&call_result_str)
            .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?;
        println!("--------------");
        println!();
        println!("{}", serde_json::to_string_pretty(&serde_call_result)?);
        Ok(())
    }
}
