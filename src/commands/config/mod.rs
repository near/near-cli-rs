use color_eyre::eyre::ContextCompat;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_connection;
mod delete_connection;
mod edit_connection;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ConfigCommands {
    #[interactive_clap(subcommand)]
    config_actions: ConfigActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What do you want to do with a near CLI config?
pub enum ConfigActions {
    #[strum_discriminants(strum(
        message = "show-connections       - Show a list of network connections"
    ))]
    /// Show a list of network connections
    ShowConnections(ShowConnections),
    #[strum_discriminants(strum(message = "add-connection         - Add a network connection"))]
    /// Add a network connection
    AddConnection(self::add_connection::AddNetworkConnection),
    #[strum_discriminants(strum(message = "edit-connection        - Edit a network connection"))]
    /// Edit a network connection
    EditConnection(self::edit_connection::EditConnection),
    #[strum_discriminants(strum(
        message = "delete-connection      - Delete a network connection"
    ))]
    /// Delete a network connection
    DeleteConnection(self::delete_connection::DeleteNetworkConnection),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ShowConnectionsContext)]
pub struct ShowConnections;

#[derive(Debug, Clone)]
pub struct ShowConnectionsContext;

impl ShowConnectionsContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<ShowConnections as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut path_config_toml =
            dirs::config_dir().wrap_err("Impossible to get your config dir!")?;
        path_config_toml.push("near-cli/config.toml");
        eprintln!(
            "\nConfiguration data is stored in a file {:?}",
            &path_config_toml
        );
        let config_toml = toml::to_string(&previous_context.config)?;
        eprintln!("{}", &config_toml);
        Ok(Self)
    }
}
