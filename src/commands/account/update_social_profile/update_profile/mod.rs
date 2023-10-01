mod profile_args_type;
mod sign_as;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    pub data: crate::types::socialdb_types::SocialDb,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = UpdateAccountProfileContext)]
pub struct UpdateAccountProfile {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to update the profile for?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    profile_args_type: self::profile_args_type::ProfileArgsType,
}

#[derive(Clone)]
pub struct UpdateAccountProfileContext {
    pub global_context: crate::GlobalContext,
    pub get_contract_account_id: super::super::storage_management::GetContractAccountId,
    pub account_id: near_primitives::types::AccountId,
}

impl UpdateAccountProfileContext {
    pub fn from_previous_context(
        previous_context: super::UpdateSocialProfileContext,
        scope: &<UpdateAccountProfile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl UpdateAccountProfile {
    pub fn input_account_id(
        context: &super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "Which account do you want to update the profile for?",
        )
    }
}
