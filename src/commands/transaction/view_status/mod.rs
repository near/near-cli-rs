use color_eyre::eyre::Context;

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the transaction you need to view:
    transaction_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
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
        let transaction_hash = scope.transaction_hash;
        let signer_account_id = scope.signer_account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let query_view_transaction_status = network_config
                        .json_rpc_client()
                        .blocking_call(near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::RpcTransactionStatusRequest {
                            transaction_info: near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo::TransactionId {
                                hash: transaction_hash.into(),
                                account_id: signer_account_id.clone().into()
                            }
                        })
                        .wrap_err("Failed to fetch query for view transaction")?;
                    eprintln!("Transaction status: {:#?}", query_view_transaction_status);
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<TransactionInfoContext> for crate::network::NetworkContext {
    fn from(item: TransactionInfoContext) -> Self {
        item.0
    }
}

impl TransactionInfo {
    pub fn input_signer_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        Ok(Some(
            crate::common::input_account_id_from_used_account_list(
                &context,
                "What is the signer account ID?",
                true,
            )?,
        ))
    }
}
