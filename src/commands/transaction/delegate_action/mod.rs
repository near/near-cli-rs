mod network_for_transaction;
mod sign_as;
mod transaction_signature_options;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the delegated action to send
    transaction_hash: String,
    #[interactive_clap(named_arg)]
    /// What is the relayer account ID?
    sign_as: self::sign_as::RelayerAccountId,
}

#[derive(Clone)]
pub struct TransactionInfoContext {
    config: crate::config::Config,
    transaction_hash: String,
}

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            transaction_hash: scope.transaction_hash.clone(),
        })
    }
}
