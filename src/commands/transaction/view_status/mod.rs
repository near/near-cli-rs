use color_eyre::eyre::Context;

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

                move |network_config| get_transaction_status(network_config, tx_hash)
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

#[tracing::instrument(name = "Getting transaction status ...", skip_all)]
fn get_transaction_status(
    network_config: &crate::config::NetworkConfig,
    tx_hash: near_primitives::hash::CryptoHash,
) -> crate::CliResult {
    let query_view_transaction_status = network_config
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
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query for view transaction on network <{}>",
                network_config.network_name
            )
        })?;
    eprintln!("\nTransaction status: {:#?}", query_view_transaction_status);
    Ok(())
}
