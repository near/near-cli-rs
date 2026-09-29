use color_eyre::eyre::Context;
use color_eyre::owo_colors::OwoColorize;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::JsonRpcClientExt;

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
                let tx_hash: near_primitives::hash::CryptoHash = scope.transaction_hash.into();

                move |network_config| {
                    let query_view_transaction_status =
                        get_transaction_info(network_config, tx_hash)?;
                    if query_view_transaction_status.final_execution_status
                        != near_primitives::views::TxExecutionStatus::Final
                    {
                        eprintln!(
                            "{}",
                            format!(
                                "The transaction is not final yet (reached: {:?}). The status below may change.",
                                query_view_transaction_status.final_execution_status
                            )
                            .yellow()
                        );
                    }
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        previous_context.verbosity
                    {
                        eprintln!("Transaction status:");
                        println!("{query_view_transaction_status:#?}");
                    } else {
                        println!("{query_view_transaction_status:?}");
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

#[tracing::instrument(name = "Getting information about transaction", skip_all)]
pub fn get_transaction_info(
    network_config: &crate::config::NetworkConfig,
    tx_hash: near_primitives::hash::CryptoHash,
) -> color_eyre::eyre::Result<near_jsonrpc_client::methods::tx::RpcTransactionResponse> {
    tracing::Span::current().pb_set_message(&format!("{tx_hash} ..."));
    tracing::info!(target: "near_teach_me", "Getting information about transaction {tx_hash} ...");
    network_config
        .json_rpc_client()
        .blocking_call(
            near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
                transaction_info:
                    near_jsonrpc_client::methods::tx::TransactionInfo::TransactionId {
                        tx_hash,
                        sender_account_id: "near".parse::<near_primitives::types::AccountId>()?,
                    },
                wait_until: near_primitives::views::TxExecutionStatus::Final,
            },
        )
        .or_else(|err| {
            crate::common::pending_transaction_status(&err)
                .cloned()
                .ok_or(err)
        })
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query for view transaction on network <{}>",
                network_config.network_name
            )
        })
}
