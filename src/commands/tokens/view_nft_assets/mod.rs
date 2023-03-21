use serde_json::json;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewNftAssetsContext)]
pub struct ViewNftAssets {
    /// What is the nft-contract account ID?
    nft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewNftAssetsContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewNftAssetsContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<ViewNftAssets as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let owner_account_id = previous_context.owner_account_id;
        let nft_contract_account_id: near_primitives::types::AccountId =
            scope.nft_contract_account_id.clone().into();

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            move |network_config, block_reference| {
                let method_name = "nft_tokens_for_owner".to_string();
                let args = json!({
                    "account_id": owner_account_id.to_string(),
                })
                .to_string()
                .into_bytes();
                let query_view_method_response = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(
                    network_config
                    .json_rpc_client()
                    .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: block_reference.clone(),
                        request: near_primitives::views::QueryRequest::CallFunction {
                            account_id: nft_contract_account_id.clone(),
                            method_name,
                            args: near_primitives::types::FunctionArgs::from(args),
                        },
                    }))
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
                    })?;
                let call_result =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                        query_view_method_response.kind
                    {
                        result.result
                    } else {
                        return Err(color_eyre::Report::msg("Error call result".to_string()));
                    };
                let serde_call_result = if call_result.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::from_slice(&call_result)
                        .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
                };

                println!("\n{} account has NFT tokens:", owner_account_id);
                println!("{}", serde_json::to_string_pretty(&serde_call_result)?);
                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewNftAssetsContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewNftAssetsContext) -> Self {
        item.0
    }
}
