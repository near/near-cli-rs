use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::SelectServerContext)]
pub struct Server {
    #[interactive_clap(named_arg)]
    ///Specify a contract
    pub account: super::super::super::super::sender::Sender,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SelectServerContext)]
#[interactive_clap(output_context = super::ViewContractCodeCommandNetworkContext)]
pub struct CustomServer {
    #[interactive_clap(long)]
    pub url: crate::common::AvailableRpcServerUrl,
    #[interactive_clap(named_arg)]
    ///Specify a contract
    pub account: super::super::super::super::sender::Sender,
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

impl From<CustomServerContext> for super::ViewContractStateCommandNetworkContext {
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
        self.account.process(connection_config).await
    }
}

impl CustomServer {
    pub fn input_url(
        _context: &super::SelectServerContext,
    ) -> color_eyre::eyre::Result<crate::common::AvailableRpcServerUrl> {
        Ok(Input::new()
            .with_prompt("What is the RPC endpoint?")
            .interact_text()
            .unwrap())
    }

    pub async fn process(self) -> crate::CliResult {
        let connection_config = crate::common::ConnectionConfig::from_custom_url(&self.url);
        self.account.process(connection_config).await
    }
}
