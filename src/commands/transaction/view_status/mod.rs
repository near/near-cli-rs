use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the transaction you need to view:
    transaction_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct TransactionInfoContext(crate::network::NetworkContext);

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let tx_hash: near_kit::CryptoHash = scope.transaction_hash.into();

                move |network_config| {
                    let query_view_transaction_status =
                        get_transaction_info(network_config, tx_hash)?;
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        previous_context.verbosity
                    {
                        eprintln!("Transaction status:");
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&query_view_transaction_status.json)
                                .unwrap_or_default()
                        );
                    } else {
                        println!("{}", query_view_transaction_status.json);
                    }
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback,
        }))
    }
}

impl From<TransactionInfoContext> for crate::network::NetworkContext {
    fn from(item: TransactionInfoContext) -> Self {
        item.0
    }
}

/// Response from get_transaction_info, containing the raw JSON from EXPERIMENTAL_tx_status.
///
/// Callers that need structured access (reconstruct_transaction) should
/// parse `final_execution_outcome` from the JSON.
#[derive(Debug)]
pub struct TransactionStatusResponse {
    pub json: serde_json::Value,
}

impl TransactionStatusResponse {
    /// Extract the final execution outcome.
    pub fn final_execution_outcome(&self) -> Option<near_kit::FinalExecutionOutcome> {
        self.json
            .get("final_execution_outcome")
            .and_then(|v| if v.is_null() { None } else { Some(v) })
            .and_then(|v| serde_json::from_value::<near_kit::FinalExecutionOutcome>(v.clone()).ok())
    }
}

impl std::fmt::Display for TransactionStatusResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", self.json)
    }
}

#[tracing::instrument(name = "Getting information about transaction", skip_all)]
pub fn get_transaction_info(
    network_config: &crate::config::NetworkConfig,
    tx_hash: near_kit::CryptoHash,
) -> color_eyre::eyre::Result<TransactionStatusResponse> {
    tracing::Span::current().pb_set_message(&format!("{tx_hash} ..."));
    tracing::info!(target: "near_teach_me", "Getting information about transaction {tx_hash} ...");

    let params = serde_json::json!({
        "tx_hash": tx_hash.to_string(),
        "sender_account_id": "near",
        "wait_until": near_kit::TxExecutionStatus::Final.as_str(),
    });

    let json: serde_json::Value = crate::common::block_on(
        network_config
            .client()
            .rpc()
            .call("EXPERIMENTAL_tx_status", params),
    )
    .map_err(|err| {
        color_eyre::eyre::eyre!(
            "Failed to fetch query for view transaction on network <{}>: {}",
            network_config.network_name,
            err
        )
    })?;

    Ok(TransactionStatusResponse { json })
}
