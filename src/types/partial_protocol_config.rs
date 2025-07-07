use near_jsonrpc_client::methods::EXPERIMENTAL_protocol_config::RpcProtocolConfigError;
use near_primitives::serialize::dec_format;

#[derive(Debug, serde::Deserialize)]
pub struct PartialProtocolConfigView {
    pub runtime_config: PartialRuntimeConfigView,
}

impl near_jsonrpc_client::methods::RpcHandlerResponse for PartialProtocolConfigView {}

#[derive(Debug, serde::Deserialize)]
pub struct PartialRuntimeConfigView {
    /// Amount of yN per byte required to have on the account.  See
    /// <https://nomicon.io/Economics/Economic#state-stake> for details.
    #[serde(with = "dec_format")]
    pub storage_amount_per_byte: near_primitives::types::Balance,
}

pub async fn get_partial_protocol_config(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<PartialProtocolConfigView> {
    let request = near_jsonrpc_client::methods::any::<
        Result<PartialProtocolConfigView, RpcProtocolConfigError>,
    >(
        "EXPERIMENTAL_protocol_config",
        serde_json::to_value(block_reference)?,
    );

    json_rpc_client
        .call(request)
        .await
        .map_err(|_| color_eyre::eyre::eyre!("Failed to get protocol config."))
}
