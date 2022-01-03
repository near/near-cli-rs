use dialoguer::Input;

use crate::common::display_proposals_info;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::NetworkContext)]
pub struct Server {}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SelectServerContext)]
#[interactive_clap(output_context = super::NetworkContext)]
pub struct CustomServer {
    #[interactive_clap(long)]
    pub url: crate::common::AvailableRpcServerUrl,
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

impl From<CustomServerContext> for super::NetworkContext {
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
        display_proposals_info(&connection_config).await?;
        Ok(())
    }
}

impl CustomServer {
    pub fn input_url(
        _context: &super::SelectServerContext,
    ) -> color_eyre::eyre::Result<crate::common::AvailableRpcServerUrl> {
        Ok(Input::new()
            .with_prompt("What is the RPC endpoint?")
            .interact_text()?)
    }

    pub async fn process(self) -> crate::CliResult {
        let connection_config = crate::common::ConnectionConfig::from_custom_url(&self.url);
        display_proposals_info(&connection_config).await?;
        Ok(())
    }
}
