#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(_context = NonceContext)]
#[interactive_clap(output_context = NonceWrapperContext)]
pub struct Nonce {
    /// A 32-byte nonce as a base64-encoded string:
    nonce: crate::types::nonce32_bytes::Nonce32,
    #[interactive_clap(named_arg)]
    recipient: super::recipient::Recipient,
}

#[derive(Debug, Clone)]
pub struct NonceContext {
    pub global_context: crate::GlobalContext,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct NonceWrapperContext(super::recipient::RecipientContext);

impl NonceWrapperContext {
    pub fn from_previous_context(
        previous_context: NonceContext,
        scope: &<Nonce as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::recipient::RecipientContext {
            global_context: previous_context.global_context,
            message: previous_context.message,
            nonce: scope.nonce.clone(),
        }))
    }
}

impl From<NonceWrapperContext> for super::recipient::RecipientContext {
    fn from(item: NonceWrapperContext) -> Self {
        item.0
    }
}
