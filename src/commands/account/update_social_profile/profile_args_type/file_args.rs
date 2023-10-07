#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = FileArgsContext)]
pub struct FileArgs {
    /// Enter the path to the input data file:
    data_path: crate::types::file_bytes::FileBytes,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct FileArgsContext(super::ArgsContext);

impl FileArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateSocialProfileContext,
        scope: &<FileArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            data: scope.data_path.read_bytes()?,
        }))
    }
}

impl From<FileArgsContext> for super::ArgsContext {
    fn from(item: FileArgsContext) -> Self {
        item.0
    }
}
