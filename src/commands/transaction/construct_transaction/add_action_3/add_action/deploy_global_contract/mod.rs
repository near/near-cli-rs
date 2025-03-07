use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = DeployGlobalContractActionContext)]
pub struct DeployGlobalContractAction {
    /// What is a file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    mode: DeployGlobalMode,
}

#[derive(Debug, Clone)]
pub struct DeployGlobalContractActionContext {
    pub context: super::super::super::ConstructTransactionContext,
    pub code: Vec<u8>,
}

impl DeployGlobalContractActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<DeployGlobalContractAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
            format!("Failed to open or read the file: {:?}.", &scope.file_path.0,)
        })?;
        Ok(Self {
            context: previous_context,
            code,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DeployGlobalContractActionContext)]
#[interactive_clap(output_context = DeployGlobalModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Choose a global contract deploy mode:
pub enum DeployGlobalMode {
    #[strum_discriminants(strum(
        message = "as-global-hash       - Deploy code as a global contract code hash (immutable)"
    ))]
    /// Deploy code as a global contract code hash (immutable)
    AsGlobalHash(NextCommand),
    #[strum_discriminants(strum(
        message = "as-global-account-id - Deploy code as a global contract account ID (mutable)"
    ))]
    /// Deploy code as a global contract account ID (mutable)
    AsGlobalAccountId(NextCommand),
}

#[derive(Debug, Clone)]
pub struct DeployGlobalModeContext(super::super::super::ConstructTransactionContext);

impl DeployGlobalModeContext {
    pub fn from_previous_context(
        previous_context: DeployGlobalContractActionContext,
        scope: &<DeployGlobalMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::DeployGlobalContract(
            near_primitives::action::DeployGlobalContractAction {
                code: previous_context.code,
                deploy_mode: match scope {
                    DeployGlobalModeDiscriminants::AsGlobalHash => {
                        near_primitives::action::GlobalContractDeployMode::CodeHash
                    }
                    DeployGlobalModeDiscriminants::AsGlobalAccountId => {
                        near_primitives::action::GlobalContractDeployMode::AccountId
                    }
                },
            },
        );
        let mut actions = previous_context.context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.context.global_context,
            signer_account_id: previous_context.context.signer_account_id,
            receiver_account_id: previous_context.context.receiver_account_id,
            actions,
        }))
    }
}

impl From<DeployGlobalModeContext> for super::super::super::ConstructTransactionContext {
    fn from(item: DeployGlobalModeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeployGlobalModeContext)]
pub struct NextCommand {
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_last::NextAction,
}
