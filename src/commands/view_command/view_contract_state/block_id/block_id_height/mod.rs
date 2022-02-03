use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl BlockIdHeight {
    pub fn input_block_id_height(
        _context: &super::super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::BlockHeight> {
        Ok(Input::new()
            .with_prompt("Type the block ID height for this account")
            .interact_text()?)
    }

    pub async fn process(
        self,
        sender_account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = near_jsonrpc_client::JsonRpcClient::connect(
            &network_connection_config.rpc_url().as_str(),
        )
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::BlockReference::BlockId(
                near_primitives::types::BlockId::Height(self.block_id_height.clone()),
            ),
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
