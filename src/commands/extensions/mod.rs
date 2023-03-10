use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod self_update;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ExtensionsCommands {
    #[interactive_clap(subcommand)]
    pub extensions_actions: ExtensionsActions,
}

impl ExtensionsCommands {
    pub async fn process(&self) -> crate::CliResult {
        self.extensions_actions.process().await
    }
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

impl ExtensionsActions {
    pub async fn process(&self) -> crate::CliResult {
        match self {
            Self::SelfUpdate(self_update_command) => self_update_command.process().await,
        }
    }
}
