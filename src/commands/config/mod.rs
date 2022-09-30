use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_connection;
mod delete_connection;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ConfigCommands {
    #[interactive_clap(subcommand)]
    config_actions: ConfigActions,
}

impl ConfigCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.config_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with a near-cli config?
pub enum ConfigActions {
    #[strum_discriminants(strum(
        message = "show-connections       - Show a list of network connections"
    ))]
    ///Show a list of network connections
    ShowConnections,
    #[strum_discriminants(strum(message = "add-connection         - Add a network connection"))]
    ///Add a network connection
    AddConnection(self::add_connection::AddNetworkConnection),
    #[strum_discriminants(strum(
        message = "delete-connection      - Delete a network connection"
    ))]
    ///Delete a network connection
    DeleteConnection(self::delete_connection::DeleteNetworkConnection),
}

impl ConfigActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::ShowConnections => {
                let mut path_config_toml =
                    dirs::config_dir().expect("Impossible to get your config dir!");
                path_config_toml.push("near-cli/config.toml");
                println!(
                    "\nConfiguration data is stored in a file {:?}",
                    &path_config_toml
                );
                let config_toml = toml::to_string(&config)?;
                println!("{}", &config_toml);
                Ok(())
            }
            Self::AddConnection(add_network_connection) => {
                add_network_connection.process(config).await
            }
            Self::DeleteConnection(delete_network_connection) => {
                delete_network_connection.process(config).await
            }
        }
    }
}
