use dialoguer::Input;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::display_validators_info;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct OperationMode {
    #[interactive_clap(named_arg)]
    /// Prepare and, optionally, submit a new transaction with online mode
    pub network: NetworkArgs,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct NetworkArgs {
    #[interactive_clap(subcommand)]
    selected_server: SelectServer,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(input_context = ())]
#[interactive_clap(output_context = SelectServerContext)]
///Select NEAR protocol RPC server
pub enum SelectServer {
    /// Provide data for the server https://rpc.testnet.near.org
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(Server),
    /// Provide data for the server https://rpc.mainnet.near.org
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(Server),
    /// Provide data for the server https://rpc.betanet.near.org
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(Server),
    /// Provide data for a manually specified server
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(CustomServer),
}

#[derive(Clone)]
pub struct SelectServerContext {
    selected_server: SelectServerDiscriminants,
}

impl SelectServerContext {
    fn from_previous_context(
        _previous_context: (),
        scope: &<SelectServer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            selected_server: scope.clone(),
        }
    }
}

#[derive(Clone)]
pub struct NetworkContext {
    pub connection_config: crate::common::ConnectionConfig,
}

impl From<SelectServerContext> for NetworkContext {
    fn from(item: SelectServerContext) -> Self {
        let connection_config = match item.selected_server {
            SelectServerDiscriminants::Testnet => crate::common::ConnectionConfig::Testnet,
            SelectServerDiscriminants::Mainnet => crate::common::ConnectionConfig::Mainnet,
            SelectServerDiscriminants::Betanet => crate::common::ConnectionConfig::Betanet,
            SelectServerDiscriminants::Custom => {
                unreachable!("Network context should not be constructed from Custom variant")
            }
        };
        Self { connection_config }
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
pub struct Server {
    #[interactive_clap(subcommand)]
    ///Specify an epoch
    pub epoch: EpochCommand,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = SelectServerContext)]
#[interactive_clap(output_context = NetworkContext)]
pub struct CustomServer {
    #[interactive_clap(long)]
    pub url: crate::common::AvailableRpcServerUrl,
    #[interactive_clap(subcommand)]
    ///Specify an epoch
    pub epoch: EpochCommand,
}

struct CustomServerContext {
    pub url: crate::common::AvailableRpcServerUrl,
}

impl CustomServerContext {
    fn from_previous_context(
        _previous_context: SelectServerContext,
        scope: &<CustomServer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            url: scope.url.clone(),
        }
    }
}

impl From<CustomServerContext> for NetworkContext {
    fn from(item: CustomServerContext) -> Self {
        Self {
            connection_config: crate::common::ConnectionConfig::from_custom_url(&item.url),
        }
    }
}

impl CustomServer {
    pub fn input_url(
        _context: &SelectServerContext,
    ) -> color_eyre::eyre::Result<crate::common::AvailableRpcServerUrl> {
        Ok(Input::new()
            .with_prompt("What is the RPC endpoint?")
            .interact_text()?)
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = NetworkContext)]
///Choose Block ID
pub enum EpochCommand {
    #[strum_discriminants(strum(message = "View latest validators"))]
    /// Specify latest validators
    Latest,
    // #[strum_discriminants(strum(
    //     message = "View validators by EpochId"
    // ))]
    // EpochId(self::view_command::ViewQueryRequest),
    #[strum_discriminants(strum(message = "View validators by BlockId"))]
    /// Specify validators by BlockId
    BlockId(BlockIdWrapper),
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
pub struct BlockIdWrapper {
    #[interactive_clap(subcommand)]
    block_id: BlockId,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = NetworkContext)]
///Choose Block ID
pub enum BlockId {
    #[strum_discriminants(strum(message = "View validators at final block"))]
    /// Specify a block ID final to view validators
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View validators at block heigt"))]
    /// Specify a block ID height to view validators
    AtBlockHeight(BlockIdHeight),
    #[strum_discriminants(strum(message = "View validators at block hash"))]
    /// Specify a block ID hash to view validators
    AtBlockHash(BlockIdHash),
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl BlockIdHeight {
    pub fn input_block_id_height(
        _context: &NetworkContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::BlockHeight> {
        Ok(Input::new()
            .with_prompt("Type the block ID height to view validators")
            .interact_text()?)
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
pub struct BlockIdHash {
    block_id_hash: crate::types::crypto_hash::CryptoHash,
}

impl BlockIdHash {
    pub fn input_block_id_hash(
        _context: &NetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
        Ok(Input::new()
            .with_prompt("Type the block ID hash to view validators")
            .interact_text()?)
    }
}

impl OperationMode {
    pub async fn process(self) -> crate::CliResult {
        self.network.process().await
    }
}

impl NetworkArgs {
    pub async fn process(self) -> crate::CliResult {
        self.selected_server.process().await
    }
}

impl SelectServer {
    pub async fn process(self) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Testnet;
                server.process(connection_config).await?;
            }
            SelectServer::Mainnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Mainnet;
                server.process(connection_config).await?;
            }
            SelectServer::Betanet(server) => {
                let connection_config = crate::common::ConnectionConfig::Betanet;
                server.process(connection_config).await?;
            }
            SelectServer::Custom(custom_server) => {
                custom_server.process().await?;
            }
        })
    }
}

impl Server {
    pub async fn process(
        self,
        connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.epoch.process(connection_config).await
    }
}

impl CustomServer {
    pub async fn process(self) -> crate::CliResult {
        let connection_config = crate::common::ConnectionConfig::from_custom_url(&self.url);
        self.epoch.process(connection_config).await
    }
}

impl EpochCommand {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            Self::Latest => {
                display_validators_info(
                    near_primitives::types::EpochReference::Latest,
                    &network_connection_config,
                )
                .await?;
                Ok(())
            }
            // Self::EpochId(validators_request) => validators_request.process().await,
            Self::BlockId(validators_request) => {
                validators_request.process(network_connection_config).await
            }
        }
    }
}

impl BlockIdWrapper {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.block_id.process(network_connection_config).await
    }
}

impl BlockId {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height.process(network_connection_config).await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash.process(network_connection_config).await
            }
            Self::AtFinalBlock => {
                display_validators_info(
                    near_primitives::types::EpochReference::Latest,
                    &network_connection_config,
                )
                .await?;
                Ok(())
            }
        }
    }
}

impl BlockIdHeight {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        display_validators_info(
            near_primitives::types::EpochReference::BlockId(
                near_primitives::types::BlockId::Height(self.block_id_height),
            ),
            &network_connection_config,
        )
        .await?;
        Ok(())
    }
}

impl BlockIdHash {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        display_validators_info(
            near_primitives::types::EpochReference::BlockId(near_primitives::types::BlockId::Hash(
                self.block_id_hash.into(),
            )),
            &network_connection_config,
        )
        .await?;
        Ok(())
    }
}
