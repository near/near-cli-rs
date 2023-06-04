use color_eyre::eyre::Context;

mod initialize_mode;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::ConstructTransactionContext)]
pub struct DeployContractAction {
    #[interactive_clap(named_arg)]
    /// Specify a path to wasm file
    use_file: ContractFile,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is a file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

#[derive(Clone)]
pub struct ContractFileContext(super::super::super::ConstructTransactionContext);

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
            format!("Failed to open or read the file: {:?}.", &scope.file_path.0,)
        })?;
        let action = near_primitives::transaction::Action::DeployContract(
            near_primitives::transaction::DeployContractAction { code },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<ContractFileContext> for super::super::super::ConstructTransactionContext {
    fn from(item: ContractFileContext) -> Self {
        item.0
    }
}
