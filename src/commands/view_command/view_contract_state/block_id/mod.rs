use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext)]
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
        sender_account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(sender_account_id, network_connection_config)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(sender_account_id, network_connection_config)
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(sender_account_id, network_connection_config)
                    .await
            }
        }
    }

    async fn at_final_block(
        self,
        sender_account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = near_jsonrpc_client::JsonRpcClient::connect(
            &network_connection_config.rpc_url().as_str(),
        )
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::ViewState {
                account_id: sender_account_id,
                prefix: near_primitives::types::StoreKey::from(vec![]),
            },
        })
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to fetch query for view account: {:?}", err))
        })?;
        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        println!(
            "\nContract state (values):\n{:#?}\n",
            &call_access_view.values
        );
        println!(
            "\nContract state (proof):\n{:#?}\n",
            &call_access_view.proof
        );
        Ok(())
    }
}
