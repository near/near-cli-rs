#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct DeleteNetworkConnection {
    ///What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

impl DeleteNetworkConnection {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(context)
    }

    pub async fn process(&self, mut config: crate::config::Config) -> crate::CliResult {
        config.networks.remove(&self.connection_name);
        println!();
        crate::common::write_config_toml(config)?;
        println!(
            "Network connection \"{}\" was successfully removed from config.toml",
            &self.connection_name
        );
        Ok(())
    }
}
