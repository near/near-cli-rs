mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SendMetaTransactionContext)]
pub struct SendMetaTransaction {
    /// Enter a signed delegate action as base64-encoded string:
    signed_delegate_action: crate::types::signed_delegate_action::SignedDelegateActionAsBase64,
    #[interactive_clap(named_arg)]
    /// What is the relayer account ID?
    sign_as: self::sign_as::RelayerAccountId,
}

#[derive(Clone)]
pub struct SendMetaTransactionContext {
    global_context: crate::GlobalContext,
    signed_delegate_action: near_primitives::delegate_action::SignedDelegateAction,
}

impl SendMetaTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SendMetaTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            signed_delegate_action: scope.signed_delegate_action.inner.clone(),
        })
    }
}
