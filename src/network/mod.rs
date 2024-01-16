#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NetworkContext)]
#[interactive_clap(output_context = NetworkOutputContext)]
pub struct Network {
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    wallet_url: Option<crate::types::url::Url>,
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
}

pub type OnAfterGettingNetworkCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig) -> crate::CliResult>;

#[derive(Clone)]
pub struct NetworkContext {
    pub config: crate::config::Config,
    pub interacting_with_account_ids: Vec<near_primitives::types::AccountId>,
    pub on_after_getting_network_callback: OnAfterGettingNetworkCallback,
}

#[derive(Clone)]
pub struct NetworkOutputContext;

impl NetworkOutputContext {
    pub fn from_previous_context(
        previous_context: NetworkContext,
        scope: &<Network as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_connection = previous_context.config.network_connection;
        let mut network_config = network_connection
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        if let Some(url) = scope.wallet_url.clone() {
            network_config.wallet_url = url.into();
        }

        (previous_context.on_after_getting_network_callback)(&network_config)?;
        Ok(Self)
    }
}

impl Network {
    fn input_network_name(context: &NetworkContext) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &context.interacting_with_account_ids)
    }
}
