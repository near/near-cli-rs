#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SignTransactionContext)]
pub struct SignTransaction {
    /// Enter the transaction encoded in base64:
    unsigned_transaction: crate::types::transaction::TransactionAsBase64,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignTransactionContext(crate::commands::ActionContext);

impl SignTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SignTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let unsigned_transaction: near_primitives::transaction::Transaction =
                    scope.unsigned_transaction.clone().into();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction::from(
                        unsigned_transaction.clone(),
                    ))
                }
            });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context,
            interacting_with_account_ids: vec![
                scope.unsigned_transaction.inner.signer_id().clone(),
                scope.unsigned_transaction.inner.receiver_id().clone(),
            ],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }))
    }
}

impl From<SignTransactionContext> for crate::commands::ActionContext {
    fn from(item: SignTransactionContext) -> Self {
        item.0
    }
}
