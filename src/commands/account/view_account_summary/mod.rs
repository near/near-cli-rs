#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewAccountSummaryContext)]
pub struct ViewAccountSummary {
    /// What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewAccountSummaryContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewAccountSummaryContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewAccountSummary as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = scope.account_id.clone();

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            move |network_config, block_reference| {


                let resp = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(
                    network_config
                .json_rpc_client()
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: block_reference.clone(),
                    request: near_primitives::views::QueryRequest::ViewAccount {
                        account_id: account_id.clone().into(),
                    },
                })
            )
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

            let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(
                network_config
                .json_rpc_client()
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::BlockId::Hash(resp.block_hash).into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                        account_id: account_id.clone().into(),
                    },
                })
            )
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
                &account_id,
                &account_view,
                &access_keys,
            );
            Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.0,
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewAccountSummaryContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewAccountSummaryContext) -> Self {
        item.0
    }
}
