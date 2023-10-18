#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = JsonArgsContext)]
pub struct JsonArgs {
    /// Enter valid JSON arguments (e.g. {\"token_id\": \"42\"})":
    data: crate::types::json::Json,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct JsonArgsContext(super::ArgsContext);

impl JsonArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateSocialProfileContext,
        scope: &<JsonArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            data: scope.data.try_into_bytes()?,
        }))
    }
}

impl From<JsonArgsContext> for super::ArgsContext {
    fn from(item: JsonArgsContext) -> Self {
        item.0
    }
}
