#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LinkdropAccountIdContext)]
pub struct LinkdropAccountId {
    /// What is the name of the account that hosts the "linkdrop" program? (e.g. on mainnet it is near, and on testnet it is testnet)
    linkdrop_account_id: crate::types::account_id::AccountId,
    /// What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

#[derive(Debug, Clone)]
pub struct LinkdropAccountIdContext;

impl LinkdropAccountIdContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LinkdropAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        if let Some(network_config) = config.network_connection.get_mut(&scope.connection_name) {
            network_config.linkdrop_account_id = Some(scope.linkdrop_account_id.clone().into());
        } else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &scope.connection_name
            ));
        };
        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Linkdrop account ID successfully updated for Network connection \"{}\"",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl LinkdropAccountId {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}
