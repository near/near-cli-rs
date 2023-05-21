#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessKeyPermissionContext)]
#[interactive_clap(output_context = AddAccessKeyActionContext)]
pub struct AddAccessKeyAction {
    /// Enter the public key for this account:
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::super::add_action_last::NextAction,
}

#[derive(Clone)]
pub struct AddAccessKeyActionContext(super::super::super::super::ConstructTransactionContext);

impl AddAccessKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessKeyPermissionContext,
        scope: &<AddAccessKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission: previous_context.access_key_permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: scope.public_key.clone().into(),
                access_key,
            },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(
            super::super::super::super::ConstructTransactionContext {
                config: previous_context.config,
                signer_account_id: previous_context.signer_account_id,
                receiver_account_id: previous_context.receiver_account_id,
                actions,
            },
        ))
    }
}

impl From<AddAccessKeyActionContext> for super::super::super::super::ConstructTransactionContext {
    fn from(item: AddAccessKeyActionContext) -> Self {
        item.0
    }
}
