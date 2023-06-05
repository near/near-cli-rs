mod network;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SignedTransactionContext)]
pub struct SignedTransaction {
    /// Enter a signed transaction as base64-encoded string:
    signed_action: crate::types::signed_transaction::SignedTransactionAsBase64,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: self::network::Network,
}

#[derive(Debug, Clone)]
pub struct SignedTransactionContext {
    config: crate::config::Config,
    signed_transaction: near_primitives::transaction::SignedTransaction,
}

impl SignedTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SignedTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signed_transaction: scope.signed_action.inner.clone(),
        })
    }
}
