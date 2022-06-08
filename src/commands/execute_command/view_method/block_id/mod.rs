use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext)]
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
        contract_account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(
                        network_connection_config,
                        contract_account_id,
                        method_name,
                        args,
                    )
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(
                        network_connection_config,
                        contract_account_id,
                        method_name,
                        args,
                    )
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(
                    network_connection_config,
                    contract_account_id,
                    method_name,
                    args,
                )
                .await
            }
        }
    }

    async fn at_final_block(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        contract_account_id: near_primitives::types::AccountId,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        let args: near_primitives::types::FunctionArgs =
            near_primitives::types::FunctionArgs::from(args);
        let query_view_method_response =
            near_jsonrpc_client::JsonRpcClient::connect(network_connection_config.rpc_url())
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
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
        let call_result =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                query_view_method_response.kind
            {
                result.result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };

        let serde_call_result = if call_result.is_empty() {
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
