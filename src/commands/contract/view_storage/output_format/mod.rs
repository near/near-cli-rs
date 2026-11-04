use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

mod as_json;
mod as_text;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::keys_to_view::KeysContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a format to view contract storage state:
pub enum OutputFormat {
    #[strum_discriminants(strum(
        message = "as-json    - View contract storage state in JSON format"
    ))]
    /// View contract storage state in JSON format
    AsJson(self::as_json::AsJson),
    #[strum_discriminants(strum(message = "as-text    - View contract storage state in the text"))]
    /// View contract storage state in the text
    AsText(self::as_text::AsText),
}

#[tracing::instrument(name = "Obtaining the state of the contract ...", skip_all)]
pub fn get_contract_state(
    contract_account_id: &near_primitives::types::AccountId,
    prefix: near_primitives::types::StoreKey,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<near_primitives::views::StateItem>> {
    tracing::info!(target: "near_teach_me", "Obtaining the state of the contract ...");
    let mut block_reference = block_reference;
    let mut after_key = None;
    let mut values = Vec::new();
    // Paginated requests skip the node's ~50 KB per-account `view_state` limit.
    loop {
        let response = network_config
            .json_rpc_client()
            .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::ViewState {
                    account_id: contract_account_id.clone(),
                    prefix: prefix.clone(),
                    include_proof: false,
                    after_key,
                    limit: std::num::NonZeroU32::new(10_000),
                },
            })
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch query ViewState for <{contract_account_id}> on network <{}>",
                    network_config.network_name
                )
            })?;
        let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(page) =
            response.kind
        else {
            return Err(color_eyre::Report::msg("Error call result"));
        };
        values.extend(page.values);
        let Some(last_key) = page.last_key else {
            return Ok(values);
        };
        after_key = Some(last_key);
        // Read the remaining pages at the same block as the first.
        block_reference = near_primitives::types::BlockId::Hash(response.block_hash).into();
    }
}
