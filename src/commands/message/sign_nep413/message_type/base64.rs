#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::MessageTypeContext)]
#[interactive_clap(output_context = Base64Context)]
pub struct Base64 {
    /// The base64-encoded message to sign:
    message: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(named_arg)]
    nonce: super::super::nonce::Nonce,
}

#[derive(Debug, Clone)]
pub struct Base64Context(super::super::nonce::NonceContext);

impl Base64Context {
    pub fn from_previous_context(
        previous_context: super::MessageTypeContext,
        scope: &<Base64 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let message = String::from_utf8(scope.message.as_bytes().to_vec())
            .map_err(|e| color_eyre::eyre::eyre!("Message is not valid UTF-8: {}", e))?;

        Ok(Self(super::super::nonce::NonceContext {
            global_context: previous_context.global_context,
            message,
        }))
    }
}

impl From<Base64Context> for super::super::nonce::NonceContext {
    fn from(item: Base64Context) -> Self {
        item.0
    }
}
