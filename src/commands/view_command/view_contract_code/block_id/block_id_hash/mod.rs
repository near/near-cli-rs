use dialoguer::Input;
use std::io::Write;

/// Specify the block_id hash for this contract to view
#[derive(Debug, Default, clap::Clap)]
pub struct CliBlockIdHash {
    block_id_hash: Option<near_primitives::hash::CryptoHash>,
}

#[derive(Debug)]
pub struct BlockIdHash {
    block_id_hash: near_primitives::hash::CryptoHash,
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
        contract_id: String,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockReference::BlockId(
                    near_primitives::types::BlockId::Hash(self.block_id_hash.clone()),
                ),
                request: near_primitives::views::QueryRequest::ViewCode {
                    account_id: contract_id,
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view contract: {:?}",
                    err
                ))
            })?;
        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        match &file_path {
            Some(file_path) => {
                std::fs::File::create(file_path)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                    })?
                    .write(&call_access_view.code)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                    })?;
                println!("\nThe file {:?} was downloaded successfully", file_path);
            }
            None => {
                println!("\nHash of the contract: {}", &call_access_view.hash)
            }
        }
        Ok(())
    }
}
