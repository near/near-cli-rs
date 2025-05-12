use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::super::super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the need for initialization:
pub enum InitializeMode {
    /// Add an initialize
    #[strum_discriminants(strum(message = "with-init-call     - Add an initialize"))]
    WithInitCall(super::super::call_function::FunctionCallAction),
    /// Don't add an initialize
    #[strum_discriminants(strum(message = "without-init-call  - Don't add an initialize"))]
    WithoutInitCall(NoInitialize),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::super::ConstructTransactionContext)]
pub struct NoInitialize {
    #[interactive_clap(subcommand)]
    next_action: super::super::super::super::add_action_last::NextAction,
}
