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
/// Callers that need structured access can deserialize the flattened outcome
/// fields through near-kit's transaction response type.
#[derive(Debug)]
pub struct TransactionStatusResponse {
    pub json: serde_json::Value,
}

impl TransactionStatusResponse {
    /// Extract the final execution outcome.
    pub fn final_execution_outcome(&self) -> Option<near_kit::FinalExecutionOutcome> {
        serde_json::from_value::<near_kit::RawTransactionResponse>(self.json.clone())
            .ok()?
            .outcome
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

#[cfg(test)]
mod tests {
    use super::TransactionStatusResponse;

    #[test]
    fn parses_flattened_top_level_execution_outcome() {
        let response = TransactionStatusResponse {
            json: serde_json::json!({
                "final_execution_status": "FINAL",
                "status": {"SuccessValue": ""},
                "transaction": {
                    "signer_id": "alice.near",
                    "public_key": "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp",
                    "nonce": 1,
                    "receiver_id": "bob.near",
                    "actions": [],
                    "signature": "ed25519:3s1dvMqNDCByoMnDnkhB4GPjTSXCRt4nt3Af5n1RX8W7aJ2FC6MfRf5BNXZ52EBifNJnNVBsGvke6GRYuaEYJXt5",
                    "hash": "9FtHUFBQsZ2MG77K3x3MJ9wjX3UT8zE1TczCrhZEcG8U"
                },
                "transaction_outcome": {
                    "id": "9FtHUFBQsZ2MG77K3x3MJ9wjX3UT8zE1TczCrhZEcG8U",
                    "outcome": {
                        "executor_id": "alice.near",
                        "gas_burnt": 0,
                        "tokens_burnt": "0",
                        "logs": [],
                        "receipt_ids": [],
                        "status": {"SuccessValue": ""}
                    },
                    "block_hash": "A6DJpKBhmAMmBuQXtY3dWbo8dGVSQ9yH7BQSJBfn8rBo",
                    "proof": []
                },
                "receipts_outcome": []
            }),
        };

        let outcome = response.final_execution_outcome().unwrap();
        assert!(outcome.is_success());
        assert_eq!(outcome.transaction.signer_id.as_str(), "alice.near");
    }
}
