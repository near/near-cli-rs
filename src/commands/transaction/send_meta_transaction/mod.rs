mod network_for_transaction;
mod sign_as;
mod transaction_signature_options;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct SendMetaTransaction {
    /// Enter a signed delegate action as base64-encoded string
    signed_delegate_action:
        crate::types::signed_delegate_action_as_base64::SignedDelegateActionAsBase64,
    #[interactive_clap(named_arg)]
    /// What is the relayer account ID?
    sign_as: self::sign_as::RelayerAccountId,
}

#[derive(Clone)]
pub struct TransactionInfoContext {
    config: crate::config::Config,
    signed_delegate_action: near_primitives::delegate_action::SignedDelegateAction,
}

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SendMetaTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            signed_delegate_action: scope.signed_delegate_action.inner.clone(),
        })
    }
}
