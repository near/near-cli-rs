use dialoguer::Input;

/// Specify the block_id height for this contract to view
#[derive(Debug, Default, clap::Clap)]
pub struct CliBlockIdHeight {
    block_id_height: Option<near_primitives::types::BlockHeight>,
}

#[derive(Debug)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
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
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        sender_account_id: String,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
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
