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
) -> color_eyre::eyre::Result<near_jsonrpc_client::methods::query::RpcQueryResponse> {
    tracing::info!(target: "near_teach_me", "Obtaining the state of the contract ...");
    network_config
        .json_rpc_client()
        .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference,
            request: near_primitives::views::QueryRequest::ViewState {
                account_id: contract_account_id.clone(),
                prefix,
                include_proof: false,
            },
        })
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query ViewState for <{contract_account_id}> on network <{}>",
                network_config.network_name
            )
        })
}
