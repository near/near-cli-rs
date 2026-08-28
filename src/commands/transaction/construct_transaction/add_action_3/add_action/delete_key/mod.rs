#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = DeleteKeyActionContext)]
pub struct DeleteKeyAction {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the public keys you wish to delete (separated by comma):
    public_keys: crate::types::public_key_list::PublicKeyList,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_last::NextAction,
}

#[derive(Debug, Clone)]
pub struct DeleteKeyActionContext(super::super::super::ConstructTransactionContext);

impl DeleteKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<DeleteKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let public_keys: Vec<near_kit::PublicKey> = scope.public_keys.clone().into();
        let action: Vec<near_kit::Action> = public_keys
            .into_iter()
            .map(|public_key| near_kit::Action::DeleteKey(near_kit::DeleteKeyAction { public_key }))
            .collect();
        let mut actions = previous_context.actions;
        actions.extend(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
            sign_as_delegate_action: previous_context.sign_as_delegate_action,
        }))
    }
}

impl From<DeleteKeyActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: DeleteKeyActionContext) -> Self {
        item.0
    }
}

impl DeleteKeyAction {
    pub fn input_public_keys(
        context: &super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key_list::PublicKeyList>> {
        crate::commands::account::delete_key::public_keys_to_delete::PublicKeyList::input_public_keys(
            &context.clone().into(),
        )
    }
}
