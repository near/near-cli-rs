#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = TransferActionContext)]
pub struct TransferAction {
    /// How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)
    amount_in_near: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_last::NextAction,
}

#[derive(Debug, Clone)]
pub struct TransferActionContext(super::super::super::ConstructTransactionContext);

impl TransferActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<TransferAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: scope.amount_in_near.to_yoctonear(),
            },
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<TransferActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: TransferActionContext) -> Self {
        item.0
    }
}
