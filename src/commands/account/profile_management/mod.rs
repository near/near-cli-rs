use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view_profile;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// For which contract account ID do you want to manage the profile?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    actions: Actions,
}

#[derive(Clone)]
pub struct ContractContext {
    pub global_context: crate::GlobalContext,
    pub get_contract_account_id: super::storage_management::GetContractAccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_account_id = scope.contract_account_id.clone();
        let get_contract_account_id: super::storage_management::GetContractAccountId =
            std::sync::Arc::new(move |_network_config| Ok(contract_account_id.clone().into()));
        Ok(Self {
            global_context: previous_context,
            get_contract_account_id,
        })
    }
}

impl Contract {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "For which contract account ID do you want to manage the profile?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ContractContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with the profile?
pub enum Actions {
    #[strum_discriminants(strum(message = "view-profile    - View profile for an account"))]
    /// View profile for an account
    ViewProfile(self::view_profile::Account),
    #[strum_discriminants(strum(message = "update-profile  - Update profile for the account"))]
    /// Update profile for the account
    UpdateProfile,
}
