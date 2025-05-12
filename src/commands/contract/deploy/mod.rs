use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod initialize_mode;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    /// Specify a deploy mode
    deploy_mode: DeployModes,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ContractContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Choose a contract action:
pub enum DeployModes {
    #[strum_discriminants(strum(
        message = "use-file              - Deploy a contract using a wasm file"
    ))]
    /// Deploy a contract using a wasm file
    UseFile(ContractFile),

    #[strum_discriminants(strum(
        message = "use-global-hash       - Deploy a contract using a global contract code hash"
    ))]
    /// Deploy a contract using a global contract code hash
    UseGlobalHash(ContractHash),

    #[strum_discriminants(strum(
        message = "use-global-account-id - Deploy a contract using an account ID"
    ))]
    /// Deploy a contract using an global contract account ID
    UseGlobalAccountId(ContractAccountId),
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

#[derive(Debug, Clone)]
pub struct GenericDeployContext {
    pub global_context: crate::GlobalContext,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub signer_account_id: near_primitives::types::AccountId,
    pub deploy_action: near_primitives::transaction::Action,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is the file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

pub struct ContractFileContext(GenericDeployContext);

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path)
            .wrap_err(sysexits::ExitCode::NoInput)
            .wrap_err_with(|| {
                format!("Failed to open or read the file: {:?}.", &scope.file_path.0,)
            })?;

        Ok(Self(GenericDeployContext {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            deploy_action: near_primitives::transaction::Action::DeployContract(
                near_primitives::action::DeployContractAction { code },
            ),
        }))
    }
}

impl From<ContractFileContext> for GenericDeployContext {
    fn from(item: ContractFileContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractHashContext)]
pub struct ContractHash {
    /// What is a global contract code hash?
    pub hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

pub struct ContractHashContext(GenericDeployContext);

impl ContractHashContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractHash as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(GenericDeployContext {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            deploy_action: near_primitives::transaction::Action::UseGlobalContract(Box::new(
                near_primitives::action::UseGlobalContractAction {
                    contract_identifier:
                        near_primitives::action::GlobalContractIdentifier::CodeHash(
                            scope.hash.into(),
                        ),
                },
            )),
        }))
    }
}

impl From<ContractHashContext> for GenericDeployContext {
    fn from(item: ContractHashContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractAccountIdContext)]
pub struct ContractAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is a global contract account ID?
    pub global_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

impl ContractAccountId {
    pub fn input_global_contract_account_id(
        context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is a global contract account ID?",
        )
    }
}

pub struct ContractAccountIdContext(GenericDeployContext);

impl ContractAccountIdContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<GenericDeployContext> {
        Ok(GenericDeployContext {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            deploy_action: near_primitives::transaction::Action::UseGlobalContract(Box::new(
                near_primitives::action::UseGlobalContractAction {
                    contract_identifier:
                        near_primitives::action::GlobalContractIdentifier::AccountId(
                            scope.global_contract_account_id.clone().into(),
                        ),
                },
            )),
        })
    }
}

impl From<ContractAccountIdContext> for GenericDeployContext {
    fn from(item: ContractAccountIdContext) -> Self {
        item.0
    }
}
