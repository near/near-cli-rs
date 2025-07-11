#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = RecipientContext)]
#[interactive_clap(output_context = RecipientWrapperContext)]
pub struct Recipient {
    /// The recipient of the message (e.g. "alice.near" or "myapp.com"):
    recipient: String,
    #[interactive_clap(named_arg)]
    sign_as: super::signer::SignAs,
}

#[derive(Debug, Clone)]
pub struct RecipientContext {
    pub global_context: crate::GlobalContext,
    pub message: String,
    pub nonce: crate::types::nonce32_bytes::Nonce32,
}

#[derive(Debug, Clone)]
pub struct RecipientWrapperContext(super::signer::SignAsContext);

impl RecipientWrapperContext {
    pub fn from_previous_context(
        previous_context: RecipientContext,
        scope: &<Recipient as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::signer::SignAsContext {
            global_context: previous_context.global_context,
            message: previous_context.message,
            nonce: previous_context.nonce,
            recipient: scope.recipient.clone(),
        }))
    }
}

impl From<RecipientWrapperContext> for super::signer::SignAsContext {
    fn from(item: RecipientWrapperContext) -> Self {
        item.0
    }
}
