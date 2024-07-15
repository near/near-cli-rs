#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = WalletUrlContext)]
pub struct WalletUrl {
    /// What is the wallet endpoint?
    wallet_url: crate::types::url::Url,
    /// What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

#[derive(Debug, Clone)]
pub struct WalletUrlContext;

impl WalletUrlContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<WalletUrl as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        if let Some(network_config) = config.network_connection.get_mut(&scope.connection_name) {
            network_config.wallet_url = scope.wallet_url.clone().into();
        } else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &scope.connection_name
            ));
        };
        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Wallet URL successfully updated for Network connection \"{}\"",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl WalletUrl {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}
