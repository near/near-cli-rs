mod profile_args_type;
mod sign_as;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    pub data: near_socialdb_client_rs::types::socialdb_types::SocialDb,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = UpdateSocialProfileContext)]
pub struct UpdateSocialProfile {
    #[interactive_clap(skip_default_input_arg)]
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    profile_args_type: self::profile_args_type::ProfileArgsType,
}

#[derive(Clone)]
pub struct UpdateSocialProfileContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
}

impl UpdateSocialProfileContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<UpdateSocialProfile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl UpdateSocialProfile {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Which account do you want to update the profile for?",
        )
    }
}
