use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
    /// Go to transaction signing
    Skip(super::skip_action::SkipAction),
}
