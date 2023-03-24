#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = DeleteAccountActionContext)]
pub struct DeleteAccountAction {
    #[interactive_clap(long)]
    /// Enter the beneficiary ID to delete this account ID
    beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_action: super::super::construct_transaction_3::NextAction,
}

#[derive(Clone)]
pub struct DeleteAccountActionContext(super::super::ConstructTransactionActionContext);

impl DeleteAccountActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        scope: &<DeleteAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_id: near_primitives::types::AccountId = scope.beneficiary_id.clone().into();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction { beneficiary_id },
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

impl From<DeleteAccountActionContext> for super::super::ConstructTransactionActionContext {
    fn from(item: DeleteAccountActionContext) -> Self {
        item.0
    }
}
