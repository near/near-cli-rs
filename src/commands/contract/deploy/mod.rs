use color_eyre::eyre::Context;

pub mod initialize_mode;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify a path to wasm file
    use_file: ContractFile,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    receiver_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            receiver_account_id: scope.account_id.clone().into(),
            signer_account_id: scope.account_id.clone().into(),
        })
    }
}

impl Contract {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is a file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

#[derive(Debug, Clone)]
pub struct ContractFileContext {
    pub global_context: crate::GlobalContext,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub signer_account_id: near_primitives::types::AccountId,
    pub code: Vec<u8>,
}

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
            format!("Failed to open or read the file: {:?}.", &scope.file_path.0,)
        })?;
        Ok(Self {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            code,
        })
    }
}
