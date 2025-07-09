#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::MessageTypeContext)]
#[interactive_clap(output_context = Utf8Context)]
pub struct Utf8 {
    /// The UTF-8 encoded message to sign:
    message: String,
    #[interactive_clap(subcommand)]
    sign_with: super::super::signature_options::SignWith,
}

#[derive(Debug, Clone)]
pub struct Utf8Context(super::super::SignNep413Context);

impl Utf8Context {
    pub fn from_previous_context(
        previous_context: super::MessageTypeContext,
        scope: &<Utf8 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut payload = previous_context.payload;
        payload.message = scope.message.clone();
        Ok(Self(super::super::SignNep413Context {
            global_context: previous_context.global_context,
            payload,
            signer_id: previous_context.signer_id,
        }))
    }
}

impl From<Utf8Context> for super::super::SignNep413Context {
    fn from(item: Utf8Context) -> Self {
        item.0
    }
}
