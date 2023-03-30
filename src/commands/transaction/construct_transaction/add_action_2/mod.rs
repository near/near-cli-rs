#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_action;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "add-action   - Select a new action"))]
    /// Choose next action
    AddAction(self::add_action::AddAction),
    #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
    /// Go to transaction signing
    Skip(super::skip_action::SkipAction),
}
