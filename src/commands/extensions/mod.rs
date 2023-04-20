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
#[non_exhaustive]
/// What do you want to do with a near CLI?
pub enum ExtensionsActions {
    #[strum_discriminants(strum(message = "self-update             - Self update near CLI"))]
    /// Self update near CLI
    SelfUpdate(self::self_update::SelfUpdateCommand),
}
