use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext)]
pub struct BlockIdHash {
    block_id_hash: crate::types::crypto_hash::CryptoHash,
}

impl BlockIdHash {
    pub fn input_block_id_hash(
        _context: &super::super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
        Ok(Input::new()
            .with_prompt("Type the block ID hash for this account")
            .interact_text()?)
    }

    pub async fn process(
        self,
        sender_account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response =
            near_jsonrpc_client::JsonRpcClient::connect(network_connection_config.archival_rpc_url())
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::BlockReference::BlockId(
                        near_primitives::types::BlockId::Hash(self.block_id_hash.clone().into()),
                    ),
                    request: near_primitives::views::QueryRequest::ViewState {
                        account_id: sender_account_id,
                        prefix: near_primitives::types::StoreKey::from(vec![]),
                    },
                })
                .await
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to fetch query for view account: {:?}",
                        err
                    ))
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
