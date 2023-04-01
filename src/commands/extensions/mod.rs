use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod self_update;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ExtensionsCommands {
    #[interactive_clap(subcommand)]
    pub extensions_actions: ExtensionsActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with a near-cli-rs?
pub enum ExtensionsActions {
    #[strum_discriminants(strum(message = "self-update             - Self update near-cli-rs"))]
    /// Self update near-cli-rs
    SelfUpdate(self::self_update::SelfUpdateCommand),
}
