#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = Base64ArgsContext)]
pub struct Base64Args {
    /// Enter valid Base64-encoded string (e.g. e30=):
    data: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct Base64ArgsContext(super::ArgsContext);

impl Base64ArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateSocialProfileContext,
        scope: &<Base64Args as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            data: scope.data.into_bytes(),
        }))
    }
}

impl From<Base64ArgsContext> for super::ArgsContext {
    fn from(item: Base64ArgsContext) -> Self {
        item.0
    }
}
