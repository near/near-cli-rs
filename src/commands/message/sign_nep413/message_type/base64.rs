#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::MessageTypeContext)]
#[interactive_clap(output_context = Base64Context)]
pub struct Base64 {
    /// The base64-encoded message to sign:
    message: String,
    #[interactive_clap(subcommand)]
    sign_with: super::super::signature_options::SignWith,
}

#[derive(Debug, Clone)]
pub struct Base64Context(super::super::SignNep413Context);

impl Base64Context {
    pub fn from_previous_context(
        previous_context: super::MessageTypeContext,
        scope: &<Base64 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut payload = previous_context.payload;
        payload.message = String::from_utf8(
            near_primitives::serialize::from_base64(&scope.message)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to decode base64 message: {}", e))?,
        )
        .map_err(|e| color_eyre::eyre::eyre!("Message is not valid UTF-8: {}", e))?;
        Ok(Self(super::super::SignNep413Context {
            global_context: previous_context.global_context,
            payload,
            signer_id: previous_context.signer_id,
        }))
    }
}

impl From<Base64Context> for super::super::SignNep413Context {
    fn from(item: Base64Context) -> Self {
        item.0
    }
}
