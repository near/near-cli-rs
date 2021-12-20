use dialoguer::Input;
use std::io::Write;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl BlockIdHeight {
    pub fn input_block_id_height(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::BlockHeight> {
        Ok(Input::new()
            .with_prompt("Type the block ID height for this account")
            .interact_text()
            .unwrap())
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockReference::BlockId(
                    near_primitives::types::BlockId::Height(self.block_id_height.clone()),
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
                let dir_name = &file_path.parent().unwrap();
                std::fs::create_dir_all(&dir_name)?;
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
