use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod rpc_url;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
pub struct Update {
    #[interactive_clap(subcommand)]
    option: NetworkConnectionOptions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you updating?
pub enum NetworkConnectionOptions {
    #[strum_discriminants(strum(message = "rpc-url     - Update the rpc url to connect"))]
    /// Update the rpc url to connect
    RpcUrl(self::rpc_url::RpcUrl),
}
