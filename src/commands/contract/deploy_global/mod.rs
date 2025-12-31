use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Contract {
    #[interactive_clap(named_arg)]
    /// Specify a path to wasm file
    use_file: ContractFile,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is the file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    mode: DeployGlobalMode,
}

#[derive(Debug, Clone)]
pub struct ContractFileContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
}

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
            format!("Failed to open or read the file: {:?}.", &scope.file_path.0,)
        })?;
        Ok(Self {
            global_context: previous_context,
            code,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractFileContext)]
#[interactive_clap(output_context = DeployGlobalModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Choose a global contract deploy mode:
pub enum DeployGlobalMode {
    #[strum_discriminants(strum(
        message = "as-global-hash       - Deploy code as a global contract code hash (immutable)"
    ))]
    /// Deploy code as a global contract code hash (immutable)
    AsGlobalHash(DeployGlobalResult),
    #[strum_discriminants(strum(
        message = "as-global-account-id - Deploy code as a global contract account ID (mutable)"
    ))]
    /// Deploy code as a global contract account ID (mutable)
    AsGlobalAccountId(DeployGlobalResult),
}

#[derive(Debug, Clone)]
pub struct DeployGlobalModeContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
    pub mode: near_primitives::action::GlobalContractDeployMode,
}

impl DeployGlobalModeContext {
    pub fn from_previous_context(
        previous_context: ContractFileContext,
        scope: &<DeployGlobalMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(DeployGlobalModeContext {
            global_context: previous_context.global_context,
            code: previous_context.code,
            mode: match scope {
                DeployGlobalModeDiscriminants::AsGlobalHash => {
                    near_primitives::action::GlobalContractDeployMode::CodeHash
                }
                DeployGlobalModeDiscriminants::AsGlobalAccountId => {
                    near_primitives::action::GlobalContractDeployMode::AccountId
                }
            },
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DeployGlobalModeContext)]
#[interactive_clap(output_context = DeployGlobalResultContext)]
pub struct DeployGlobalResult {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    account_id: crate::types::account_id::AccountId,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl DeployGlobalResult {
    pub fn input_account_id(
        context: &DeployGlobalModeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let question = match context.mode {
            near_primitives::action::GlobalContractDeployMode::CodeHash => {
                "What is the signer account ID?"
            }
            near_primitives::action::GlobalContractDeployMode::AccountId => {
                "What is the contract account ID?"
            }
        };
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            question,
        )
    }
}

pub struct DeployGlobalResultContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
    pub mode: near_primitives::action::GlobalContractDeployMode,
    pub account_id: near_primitives::types::AccountId,
}

impl DeployGlobalResultContext {
    pub fn from_previous_context(
        previous_context: DeployGlobalModeContext,
        scope: &<DeployGlobalResult as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            code: previous_context.code,
            mode: previous_context.mode,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl From<DeployGlobalResultContext> for crate::commands::ActionContext {
    fn from(item: DeployGlobalResultContext) -> Self {
        let account_id = item.account_id.clone();
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
        std::sync::Arc::new({
            move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.account_id.clone(),
                    receiver_id: item.account_id.clone(),
                    actions: vec![omni_transaction::near::types::Action::DeployGlobalContract(
                        near_primitives::action::DeployGlobalContractAction {
                            code: item.code.clone().into(),
                            deploy_mode: item.mode.clone(),
                        },
                    )],
                })
            }
        });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
