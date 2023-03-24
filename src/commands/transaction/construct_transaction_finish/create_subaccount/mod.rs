use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = CreateSubAccountActionContext)]
pub struct CreateSubAccountAction {
    #[interactive_clap(subcommand)]
    next_action: NextAction,
}

#[derive(Clone)]
pub struct CreateSubAccountActionContext(super::super::ConstructTransactionActionContext);

impl CreateSubAccountActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        _scope: &<CreateSubAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
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

impl From<CreateSubAccountActionContext> for super::super::ConstructTransactionActionContext {
    fn from(item: CreateSubAccountActionContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::ConstructTransactionActionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
    /// Go to transaction signing
    Skip(super::SkipAction),
}
