#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Network {
    ///What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
}

impl Network {
    fn input_network_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(context)
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        let network_config = config.networks;
        network_config
            .get(self.network_name.as_str())
            .expect("Impossible to get network name!")
            .clone()
    }
}
