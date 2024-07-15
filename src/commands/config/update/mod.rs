use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod explorer_transaction_url;
mod rpc_url;
mod wallet_url;

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
    #[strum_discriminants(strum(
        message = "rpc-url                      - Update the rpc URL to connect"
    ))]
    /// Update the rpc url to connect
    RpcUrl(self::rpc_url::RpcUrl),
    #[strum_discriminants(strum(
        message = "wallet-url                   - Update the wallet URL to connect"
    ))]
    /// Update the wallet url to connect
    WalletUrl(self::wallet_url::WalletUrl),
    #[strum_discriminants(strum(
        message = "explorer-transaction-url     - Update the explorer transaction URL to connect"
    ))]
    /// Update the explorer transaction URL to connect
    ExplorerTransactionUrl(self::explorer_transaction_url::ExplorerTransactionUrl),
}
