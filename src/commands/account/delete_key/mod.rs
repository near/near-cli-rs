mod public_keys_to_delete;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteKeysCommandContext)]
pub struct DeleteKeysCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account should you delete the access key for?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify public keys you wish to delete
    public_keys: self::public_keys_to_delete::PublicKeyList,
}

#[derive(Debug, Clone)]
pub struct DeleteKeysCommandContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
}

impl DeleteKeysCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteKeysCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone().into(),
        })
    }
}

impl DeleteKeysCommand {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Which account should you delete the access key for?",
        )
    }
}
