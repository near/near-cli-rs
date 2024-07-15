#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ExplorerTransactionUrlContext)]
pub struct ExplorerTransactionUrl {
    /// What is the transaction explorer endpoint?
    explorer_transaction_url: crate::types::url::Url,
    /// What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

#[derive(Debug, Clone)]
pub struct ExplorerTransactionUrlContext;

impl ExplorerTransactionUrlContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ExplorerTransactionUrl as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        if let Some(network_config) = config.network_connection.get_mut(&scope.connection_name) {
            network_config.explorer_transaction_url = scope.explorer_transaction_url.clone().into();
        } else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &scope.connection_name
            ));
        };
        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Explorer transaction URL successfully updated for Network connection \"{}\"",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl ExplorerTransactionUrl {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}
