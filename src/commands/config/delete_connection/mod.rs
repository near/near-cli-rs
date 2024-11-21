#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteNetworkConnectionContext)]
pub struct DeleteNetworkConnection {
    /// What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

#[derive(Debug, Clone)]
pub struct DeleteNetworkConnectionContext;

impl DeleteNetworkConnectionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteNetworkConnection as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        config.network_connection.remove(&scope.connection_name);
        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Network connection \"{}\" was successfully removed from config.toml",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl DeleteNetworkConnection {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}
