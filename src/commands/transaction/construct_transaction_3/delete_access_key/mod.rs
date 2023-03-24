#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = DeleteKeyActionContext)]
pub struct DeleteKeyAction {
    /// Enter the public key You wish to delete
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(subcommand)]
    next_action: super::super::construct_transaction_4::NextAction,
}

#[derive(Clone)]
pub struct DeleteKeyActionContext(super::super::ConstructTransactionActionContext);

impl DeleteKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        scope: &<DeleteKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key: scope.public_key.clone().into(),
            },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::ConstructTransactionActionContext {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<DeleteKeyActionContext> for super::super::ConstructTransactionActionContext {
    fn from(item: DeleteKeyActionContext) -> Self {
        item.0
    }
}
