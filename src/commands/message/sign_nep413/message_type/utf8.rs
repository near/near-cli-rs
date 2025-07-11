#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::MessageTypeContext)]
#[interactive_clap(output_context = Utf8Context)]
pub struct Utf8 {
    /// The text message (UTF-8 encoded) to sign:
    message: String,
    #[interactive_clap(named_arg)]
    nonce: super::super::nonce::Nonce,
}

#[derive(Debug, Clone)]
pub struct Utf8Context(super::super::nonce::NonceContext);

impl Utf8Context {
    pub fn from_previous_context(
        previous_context: super::MessageTypeContext,
        scope: &<Utf8 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::super::nonce::NonceContext {
            global_context: previous_context.global_context,
            message: scope.message.clone(),
        }))
    }
}

impl From<Utf8Context> for super::super::nonce::NonceContext {
    fn from(item: Utf8Context) -> Self {
        item.0
    }
}
