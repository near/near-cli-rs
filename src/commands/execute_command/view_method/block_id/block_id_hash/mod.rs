use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext)]
pub struct BlockIdHash {
    block_id_hash: crate::types::crypto_hash::CryptoHash,
}

impl BlockIdHash {
    pub fn input_block_id_hash(
        _context: &super::super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
        Ok(Input::new()
            .with_prompt("Type the block ID hash for this contract")
            .interact_text()?)
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
        let query_view_method_response = near_jsonrpc_client::JsonRpcClient::connect(
            &network_connection_config.rpc_url().as_str(),
        )
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::BlockReference::BlockId(
                near_primitives::types::BlockId::Hash(self.block_id_hash.clone().into()),
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
        let serde_call_result: serde_json::Value = if call_result.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&call_result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        println!("--------------");
        println!();
        println!("{}", serde_json::to_string_pretty(&serde_call_result)?);
        Ok(())
    }
}
