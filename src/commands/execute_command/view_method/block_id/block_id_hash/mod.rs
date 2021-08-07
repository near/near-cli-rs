use dialoguer::Input;

/// Specify the block_id hash for this contract to view
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliBlockIdHash {
    block_id_hash: Option<near_primitives::hash::CryptoHash>,
}

#[derive(Debug, Clone)]
pub struct BlockIdHash {
    block_id_hash: near_primitives::hash::CryptoHash,
}

impl CliBlockIdHash {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(block_id_hash) = &self.block_id_hash {
            args.push_front(block_id_hash.to_string());
        }
        args
    }
}

impl From<BlockIdHash> for CliBlockIdHash {
    fn from(block_id_hash: BlockIdHash) -> Self {
        Self {
            block_id_hash: Some(block_id_hash.block_id_hash),
        }
    }
}

impl From<CliBlockIdHash> for BlockIdHash {
    fn from(item: CliBlockIdHash) -> Self {
        let block_id_hash: near_primitives::hash::CryptoHash = match item.block_id_hash {
            Some(cli_block_id_hash) => cli_block_id_hash,
            None => BlockIdHash::input_block_id_hash(),
        };
        Self { block_id_hash }
    }
}

impl BlockIdHash {
    pub fn input_block_id_hash() -> near_primitives::hash::CryptoHash {
        Input::new()
            .with_prompt("Type the block ID hash for this contract")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        contract_account_id: String,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        let args: near_primitives::types::FunctionArgs =
            near_primitives::types::FunctionArgs::from(args);
        let query_view_method_response = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockReference::BlockId(
                    near_primitives::types::BlockId::Hash(self.block_id_hash.clone()),
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
        let call_result_str = String::from_utf8(call_result).unwrap();
        let serde_call_result: serde_json::Value = serde_json::from_str(&call_result_str)
            .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?;
        println!("--------------");
        println!();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_call_result).unwrap()
        );
        Ok(())
    }
}
