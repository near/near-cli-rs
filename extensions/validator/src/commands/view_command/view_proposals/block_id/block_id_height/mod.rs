use dialoguer::Input;
use std::io::Write;

/// Specify the block_id height for this contract to view
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliBlockIdHeight {
    block_id_height: Option<near_primitives::types::BlockHeight>,
}

#[derive(Debug, Clone)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl CliBlockIdHeight {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(block_id_height) = &self.block_id_height {
            args.push_front(block_id_height.to_string());
        }
        args
    }
}

impl From<BlockIdHeight> for CliBlockIdHeight {
    fn from(block_id_height: BlockIdHeight) -> Self {
        Self {
            block_id_height: Some(block_id_height.block_id_height),
        }
    }
}

impl From<CliBlockIdHeight> for BlockIdHeight {
    fn from(item: CliBlockIdHeight) -> Self {
        let block_id_height: near_primitives::types::BlockHeight = match item.block_id_height {
            Some(cli_block_id_hash) => cli_block_id_hash,
            None => BlockIdHeight::input_block_id_height(),
        };
        Self { block_id_height }
    }
}

impl BlockIdHeight {
    pub fn input_block_id_height() -> near_primitives::types::BlockHeight {
        Input::new()
            .with_prompt("Type the block ID height for this contract")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::JsonRpcClient::new().connect(&selected_server_url)
    }

    pub async fn process(
        self,
        _contract_id: near_primitives::types::AccountId,
        _network_connection_config: crate::common::ConnectionConfig,
        _file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        // TODO: rrefactor
        // let query_view_method_response = self
        //     .rpc_client(network_connection_config.archival_rpc_url().as_str())
        //     .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
        //         block_reference: near_primitives::types::BlockReference::BlockId(
        //             near_primitives::types::BlockId::Height(self.block_id_height.clone()),
        //         ),
        //         request: near_primitives::views::QueryRequest::ViewCode {
        //             account_id: contract_id,
        //         },
        //     })
        //     .await
        //     .map_err(|err| {
        //         color_eyre::Report::msg(format!(
        //             "Failed to fetch query for view contract: {:?}",
        //             err
        //         ))
        //     })?;
        // let call_access_view =
        //     if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
        //         query_view_method_response.kind
        //     {
        //         result
        //     } else {
        //         return Err(color_eyre::Report::msg(format!("Error call result")));
        //     };
        // match &file_path {
        //     Some(file_path) => {
        //         let dir_name = &file_path.parent().unwrap();
        //         std::fs::create_dir_all(&dir_name)?;
        //         std::fs::File::create(file_path)
        //             .map_err(|err| {
        //                 color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
        //             })?
        //             .write(&call_access_view.code)
        //             .map_err(|err| {
        //                 color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
        //             })?;
        //         println!("\nThe file {:?} was downloaded successfully", file_path);
        //     }
        //     None => {
        //         println!("\nHash of the contract: {}", &call_access_view.hash)
        //     }
        // }
        Ok(())
    }
}
