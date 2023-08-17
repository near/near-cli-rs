use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateAccountProfileContext)]
#[interactive_clap(output_context = FileArgsContext)]
pub struct FileArgs {
    /// Enter the path to the input data file:
    data_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct FileArgsContext(super::ArgsContext);

impl FileArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateAccountProfileContext,
        scope: &<FileArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = std::fs::read(&scope.data_path.0)
            .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &scope.data_path))?;
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            account_id: previous_context.account_id,
            data,
        }))
    }
}

impl From<FileArgsContext> for super::ArgsContext {
    fn from(item: FileArgsContext) -> Self {
        item.0
    }
}
