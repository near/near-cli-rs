mod keys_to_view;
mod output_format;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewStorageContext)]
pub struct ViewStorage {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    keys_to_view: self::keys_to_view::KeysToView,
}

#[derive(Debug, Clone)]
pub struct ViewStorageContext {
    global_context: crate::GlobalContext,
    contract_account_id: near_primitives::types::AccountId,
}

impl ViewStorageContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewStorage as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            contract_account_id: scope.contract_account_id.clone().into(),
        })
    }
}

impl ViewStorage {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}
