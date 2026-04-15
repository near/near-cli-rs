use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};


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
) -> color_eyre::eyre::Result<near_primitives::views::ViewStateResult> {
    tracing::info!(target: "near_teach_me", "Obtaining the state of the contract ...");
    let json_value = crate::common::blocking_view_state(
        network_config,
        contract_account_id,
        prefix.as_ref(),
        block_reference,
    )
    .wrap_err_with(|| {
        format!(
            "Failed to fetch query ViewState for <{contract_account_id}> on network <{}>",
            network_config.network_name
        )
    })?;
    serde_json::from_value(json_value)
        .wrap_err("Failed to parse ViewState response")
}
