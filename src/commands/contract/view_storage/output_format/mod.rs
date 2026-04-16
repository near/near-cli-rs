use base64::Engine as _;
use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::RpcResultExt;


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

/// Local ViewStateResult type to deserialize the RPC response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewStateResult {
    pub values: Vec<StateItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof: Vec<String>,
}

/// A key-value pair in contract state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateItem {
    pub key: String,
    pub value: String,
}

#[tracing::instrument(name = "Obtaining the state of the contract ...", skip_all)]
pub fn get_contract_state(
    contract_account_id: &near_kit::AccountId,
    prefix: Vec<u8>,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<ViewStateResult> {
    tracing::info!(target: "near_teach_me", "Obtaining the state of the contract ...");
    let mut params = serde_json::json!({
        "request_type": "view_state",
        "account_id": contract_account_id.to_string(),
        "prefix_base64": base64::engine::general_purpose::STANDARD.encode(&prefix),
        "include_proof": false,
    });
    if let serde_json::Value::Object(block_params) = block_reference.to_rpc_params() {
        if let serde_json::Value::Object(map) = &mut params {
            map.extend(block_params);
        }
    }
    let json_value = crate::common::block_on(
        network_config.client().rpc().call::<_, serde_json::Value>("query", params),
    )
    .into_eyre()
    .wrap_err_with(|| {
        format!(
            "Failed to fetch query ViewState for <{contract_account_id}> on network <{}>",
            network_config.network_name
        )
    })?;
    serde_json::from_value(json_value)
        .wrap_err("Failed to parse ViewState response")
}
