#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Network {
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
}

pub type OnAfterGettingNetworkCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig) -> crate::CliResult>;

#[derive(Clone)]
pub struct NetworkContext {
    pub config: crate::config::Config,
    pub on_after_getting_network_callback: OnAfterGettingNetworkCallback,
}

impl interactive_clap::FromCli for Network {
    type FromCliContext = NetworkContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        if clap_variant.network_name.is_none() {
            clap_variant.network_name = match Self::input_network_name(&context) {
                Ok(Some(network_name)) => Some(network_name),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let network_name = clap_variant.network_name.clone().expect("Unexpected error");
        let network_connection = context.config.network_connection;
        let network_config = network_connection
            .get(&network_name)
            .expect("Failed to get network config!")
            .clone();

        match (context.on_after_getting_network_callback)(&network_config) {
            Ok(_) => interactive_clap::ResultFromCli::Ok(clap_variant),
            Err(err) => interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        }
    }
}

impl Network {
    fn input_network_name(context: &NetworkContext) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config)
    }
}
