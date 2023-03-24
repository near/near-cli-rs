use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessKeyPermissionContext)]
#[interactive_clap(output_context = AddAccessWithSeedPhraseActionContext)]
pub struct AddAccessWithSeedPhraseAction {
    /// Enter the seed_phrase for this sub-account
    master_seed_phrase: String,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::construct_transaction_1::NextAction,
}

#[derive(Clone)]
pub struct AddAccessWithSeedPhraseActionContext(
    super::super::super::ConstructTransactionActionContext,
);

impl AddAccessWithSeedPhraseActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessKeyPermissionContext,
        scope: &<AddAccessWithSeedPhraseAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path_default = slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            seed_phrase_hd_path_default,
            &scope.master_seed_phrase,
        )?;
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission: previous_context.access_key_permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(
            super::super::super::ConstructTransactionActionContext {
                config: previous_context.config,
                signer_account_id: previous_context.signer_account_id,
                receiver_account_id: previous_context.receiver_account_id,
                actions,
            },
        ))
    }
}

impl From<AddAccessWithSeedPhraseActionContext>
    for super::super::super::ConstructTransactionActionContext
{
    fn from(item: AddAccessWithSeedPhraseActionContext) -> Self {
        item.0
    }
}
