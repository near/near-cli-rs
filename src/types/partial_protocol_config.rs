#[derive(Debug, serde::Deserialize)]
pub struct PartialProtocolConfigView {
    pub runtime_config: PartialRuntimeConfigView,
}

#[derive(Debug, serde::Deserialize)]
pub struct PartialRuntimeConfigView {
    /// Amount of yN per byte required to have on the account.  See
    /// <https://nomicon.io/Economics/Economic#state-stake> for details.
    pub storage_amount_per_byte: near_token::NearToken,
}

pub async fn get_partial_protocol_config(
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_kit::BlockReference,
) -> color_eyre::eyre::Result<PartialProtocolConfigView> {
    let params = block_reference.to_rpc_params();

    network_config
        .client()
        .rpc()
        .call("EXPERIMENTAL_protocol_config", params)
        .await
        .map_err(|_| color_eyre::eyre::eyre!("Failed to get protocol config."))
}
