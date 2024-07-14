use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = RpcUrlContext)]
pub struct RpcUrl {
    /// What is the RPC endpoint?
    rpc_url: crate::types::url::Url,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    rpc_api_key: Option<crate::types::api_key::ApiKey>,
    /// What is the network connection name?
    #[interactive_clap(skip_default_input_arg)]
    connection_name: String,
}

#[derive(Debug, Clone)]
pub struct RpcUrlContext;

impl RpcUrlContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<RpcUrl as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        if let Some(network_config) = config.network_connection.get_mut(&scope.connection_name) {
            network_config.rpc_url = scope.rpc_url.clone().into();
            network_config.rpc_api_key.clone_from(&scope.rpc_api_key);
        } else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &scope.connection_name
            ));
        };
        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Rpc URL successfully updated for Network connection \"{}\"",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl RpcUrl {
    fn input_rpc_api_key(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::api_key::ApiKey>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, the RPC endpoint requires API key")]
            Yes,
            #[strum(to_string = "No, the RPC endpoint does not require API key")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to input an API key?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let api_key: crate::types::api_key::ApiKey =
                CustomType::new("Enter an API key").prompt()?;
            Ok(Some(api_key))
        } else {
            Ok(None)
        }
    }

    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}
