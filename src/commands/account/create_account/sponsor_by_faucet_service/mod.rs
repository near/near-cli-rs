mod add_key;
mod network;

#[derive(Clone)]
pub struct SponsorServiceContext {
    pub config: crate::config::Config,
    pub new_account_id: crate::types::account_id::AccountId,
    pub public_key: near_crypto::PublicKey,
    pub on_after_getting_network_callback: self::network::OnAfterGettingNetworkCallback,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = NewAccountContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct NewAccountContext {
    config: crate::config::Config,
    new_account_id: crate::types::account_id::AccountId,
}

impl NewAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<NewAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            new_account_id: scope.new_account_id.clone(),
        })
    }
}

impl NewAccount {
    fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        super::fund_myself_create_account::NewAccount::input_new_account_id(context)
    }
}
