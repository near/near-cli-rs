use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::SelectServerContext)]
pub struct Server {
    #[interactive_clap(named_arg)]
    ///Specify a contract
    pub contract: super::super::super::super::contract::Contract,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SelectServerContext)]
#[interactive_clap(output_context = super::ExecuteViewMethodCommandNetworkContext)]
pub struct CustomServer {
    #[interactive_clap(skip_default_from_cli)]
    #[interactive_clap(long)]
    pub url: crate::common::AvailableRpcServerUrl,
    #[interactive_clap(named_arg)]
    ///Specify a contract
    pub contract: super::super::super::super::contract::Contract,
}

struct CustomServerContext {
    pub url: crate::common::AvailableRpcServerUrl,
}

impl CustomServerContext {
    fn from_previous_context(
        _previous_context: super::SelectServerContext,
        scope: &<CustomServer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            url: scope.url.clone(),
        }
    }
}

impl From<CustomServerContext> for super::ExecuteViewMethodCommandNetworkContext {
    fn from(item: CustomServerContext) -> Self {
        Self {
            connection_config: crate::common::ConnectionConfig::from_custom_url(&item.url),
        }
    }
}

impl Server {
    pub async fn process(
        self,
        connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.contract.process(connection_config).await
    }
}

impl CustomServer {
    fn from_cli_url(
        optional_cli_url: Option<
            <crate::common::AvailableRpcServerUrl as interactive_clap::ToCli>::CliVariant,
        >,
        context: &super::SelectServerContext,
    ) -> color_eyre::eyre::Result<crate::common::AvailableRpcServerUrl> {
        match optional_cli_url {
            Some(url) => Ok(url),
            None => {
                if let Ok(network) = std::env::var("CUSTOM_NETWORK") {
                    if let Ok(url) = network.parse() {
                        return Ok(url);
                    }
                }
                Self::input_url(context)
            }
        }
    }

    pub fn input_url(
        _context: &super::SelectServerContext,
    ) -> color_eyre::eyre::Result<crate::common::AvailableRpcServerUrl> {
        Ok(Input::new()
            .with_prompt("What is the RPC endpoint?")
            .interact_text()?)
    }

    pub async fn process(self) -> crate::CliResult {
        let connection_config = crate::common::ConnectionConfig::from_custom_url(&self.url);
        self.contract.process(connection_config).await
    }
}
