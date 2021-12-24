use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::SelectServerContext)]
pub struct Server {
    #[interactive_clap(named_arg)]
    ///Specify a validator
    pub validator: super::super::super::super::sender::Sender,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SelectServerContext)]
#[interactive_clap(output_context = super::super::super::AddAccessKeyCommandNetworkContext)]
pub struct CustomServer {
    pub url: crate::common::AvailableRpcServerUrl,
    #[interactive_clap(named_arg)]
    ///Specify a validator
    pub validator: super::super::super::super::sender::Sender,
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

impl From<CustomServerContext> for super::super::super::AddStakeProposalCommandNetworkContext {
    fn from(item: CustomServerContext) -> Self {
        Self {
            connection_config: Some(crate::common::ConnectionConfig::from_custom_url(&item.url)),
        }
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
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.validator
            .process(prepopulated_unsigned_transaction, Some(connection_config))
            .await
    }
}

impl CustomServer {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let connection_config = Some(crate::common::ConnectionConfig::from_custom_url(&self.url));
        self.validator
            .process(prepopulated_unsigned_transaction, connection_config)
            .await
    }
}
