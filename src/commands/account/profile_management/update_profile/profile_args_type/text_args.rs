#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateAccountProfileContext)]
#[interactive_clap(output_context = TextArgsContext)]
pub struct TextArgs {
    /// Input arbitrary text arguments
    data: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct TextArgsContext(super::ArgsContext);

impl TextArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateAccountProfileContext,
        scope: &<TextArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            account_id: previous_context.account_id,
            data: scope.data.clone(),
        }))
    }
}

impl From<TextArgsContext> for super::ArgsContext {
    fn from(item: TextArgsContext) -> Self {
        item.0
    }
}
