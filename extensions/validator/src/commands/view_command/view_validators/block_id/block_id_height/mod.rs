use dialoguer::Input;

/// Specify the block_id height for this account to view
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
            .with_prompt("Type the block ID height for this account")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::JsonRpcClient::new().connect(&selected_server_url)
    }

    pub async fn process(
        self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.display_account_info(account_id.clone(), &network_connection_config)
            .await?;
        Ok(())
    }

    async fn display_account_info(
        &self,
        _account_id: near_primitives::types::AccountId,
        _network_connection_config: &crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        // TODO: refactor
        // let query_view_method_response = self
        //     .rpc_client(network_connection_config.archival_rpc_url().as_str())
        //     .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
        //         block_reference: near_primitives::types::BlockReference::BlockId(
        //             near_primitives::types::BlockId::Height(self.block_id_height.clone()),
        //         ),
        //         request: near_primitives::views::QueryRequest::ViewAccount {
        //             account_id: account_id.clone(),
        //         },
        //     })
        //     .await
        //     .map_err(|err| {
        //         color_eyre::Report::msg(format!(
        //             "Failed to fetch query for view account: {:?}",
        //             err
        //         ))
        //     })?;
        // let account_view =
        //     if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(result) =
        //         query_view_method_response.kind
        //     {
        //         result
        //     } else {
        //         return Err(color_eyre::Report::msg(format!("Error call result")));
        //     };

        // println!(
        //     "Account details for '{}' at block #{} ({})\n\
        //     Native account balance: {}\n\
        //     Validator stake: {}\n\
        //     Storage used by the account: {} bytes",
        //     account_id,
        //     query_view_method_response.block_height,
        //     query_view_method_response.block_hash,
        //     crate::common::NearBalance::from_yoctonear(account_view.amount),
        //     crate::common::NearBalance::from_yoctonear(account_view.locked),
        //     account_view.storage_usage
        // );
        // if account_view.code_hash == near_primitives::hash::CryptoHash::default() {
        //     println!("Contract code is not deployed to this account.");
        // } else {
        //     println!(
        //         "Contract code SHA-256 checksum (hex): {}",
        //         hex::encode(account_view.code_hash.as_ref())
        //     );
        // }
        Ok(())
    }
}
