use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = StakeActionContext)]
pub struct StakeAction {
    stake_amount: crate::common::NearBalance,
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(subcommand)]
    next_action: NextAction,
}

#[derive(Clone)]
pub struct StakeActionContext(super::super::ConstructTransactionActionContext);

impl StakeActionContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        scope: &<StakeAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake: scope.stake_amount.to_yoctonear(),
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

impl From<StakeActionContext> for super::super::ConstructTransactionActionContext {
    fn from(item: StakeActionContext) -> Self {
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
