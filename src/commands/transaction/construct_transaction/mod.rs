mod add_action_1;
mod add_action_2;
mod add_action_3;
mod add_action_last;
mod skip_action;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ConstructTransactionContext)]
pub struct ConstructTransaction {
    /// What is the sender account ID?
    sender_account_id: crate::types::account_id::AccountId,
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_actions: self::add_action_1::NextAction,
}

#[derive(Clone)]
pub struct ConstructTransactionContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
}

impl ConstructTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ConstructTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            signer_account_id: scope.sender_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            actions: vec![],
        })
    }
}
