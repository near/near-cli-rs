#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = CreateAccountActionContext)]
pub struct CreateAccountAction {
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_2::NextAction,
}

#[derive(Clone)]
pub struct CreateAccountActionContext(super::super::super::ConstructTransactionContext);

impl CreateAccountActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        _scope: &<CreateAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
        );
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<CreateAccountActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: CreateAccountActionContext) -> Self {
        item.0
    }
}
