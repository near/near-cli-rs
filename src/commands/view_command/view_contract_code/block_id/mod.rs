use std::io::Write;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
///Choose Block ID
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this contract at final block"))]
    /// Specify a block ID final to view this contract
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this contract at block height"))]
    /// Specify a block ID height to view this contract
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this contract at block hash"))]
    /// Specify a block ID hash to view this contract
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl BlockId {
    pub async fn process(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(contract_id, network_connection_config, file_path)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(contract_id, network_connection_config, file_path)
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(contract_id, network_connection_config, file_path)
                    .await
            }
        }
    }

    async fn at_final_block(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        let query_view_method_response =
            near_jsonrpc_client::JsonRpcClient::connect(network_connection_config.rpc_url())
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
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
