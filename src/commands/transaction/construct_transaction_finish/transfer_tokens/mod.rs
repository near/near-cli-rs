use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = TransferCommandContext)]
pub struct TransferCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount to transfer
    amount_in_near: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    next_action: NextAction,
}

#[derive(Clone)]
pub struct TransferCommandContext(super::super::ConstructTransactionActionContext);

impl TransferCommandContext {
    pub fn from_previous_context(
        previous_context: super::super::ConstructTransactionActionContext,
        scope: &<TransferCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: scope.amount_in_near.to_yoctonear(),
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

impl From<TransferCommandContext> for super::super::ConstructTransactionActionContext {
    fn from(item: TransferCommandContext) -> Self {
        item.0
    }
}

impl TransferCommand {
    fn input_amount_in_near(
        _context: &super::super::ConstructTransactionActionContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        let input_amount: crate::common::NearBalance =
            CustomType::new("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                .prompt()?;
        Ok(Some(input_amount))
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
