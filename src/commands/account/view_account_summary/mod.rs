#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ViewAccountSummary {
    ///What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ViewAccountSummary {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config);

        let resp = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: self.network_config.get_block_ref(),
                request: near_primitives::views::QueryRequest::ViewAccount {
                    account_id: self.account_id.clone().into(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view account: {:?}",
                    err
                ))
            })?;

        let account_view = match resp.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(view) => view,
            _ => return Err(color_eyre::Report::msg("Error call result")),
        };

        let resp = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockId::Hash(resp.block_hash).into(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id: self.account_id.clone().into(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view key list: {:?}",
                    err
                ))
            })?;

        let access_keys = match resp.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(result) => {
                result.keys
            }
            _ => return Err(color_eyre::Report::msg("Error call result".to_string())),
        };

        crate::common::display_account_info(
            &resp.block_hash,
            &resp.block_height,
            &self.account_id,
            &account_view,
            &access_keys,
        );
        Ok(())
    }
}
