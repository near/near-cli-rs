use near_primitives::transaction;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod account;
mod config;
// mod contract;
mod tokens;
// mod transaction;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "account     - Manage accounts"))]
    /// View account summary, create subaccount, delete account, list keys, add key, delete key, import account
    Account(self::account::AccountCommands),
    #[strum_discriminants(strum(
        message = "tokens      - Manage token assets such as NEAR, FT, NFT"
    ))]
    /// Use this for token actions: send or view balances of NEAR, FT, or NFT
    Tokens(self::tokens::TokensCommands),
    // #[strum_discriminants(strum(
    //     message = "contract    - Manage smart-contracts: deploy code, call functions"
    // ))]
    // /// Use this for contract actions: call function, deploy, download wasm, inspect storage
    // Contract(self::contract::ContractCommands),
    // #[strum_discriminants(strum(message = "transaction - Operate transactions"))]
    // /// Use this to construct transactions or view a transaction status.
    // Transaction(self::transaction::TransactionCommands),
    #[strum_discriminants(strum(
        message = "config      - Manage connections in a configuration file (config.toml)"
    ))]
    /// Use this to manage connections in a configuration file (config.toml).
    Config(self::config::ConfigCommands),
}

impl TopLevelCommand {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::Tokens(tokens_commands) => tokens_commands.process(config).await,
            Self::Account(account_commands) => account_commands.process(config).await,
            // Self::Contract(contract_commands) => contract_commands.process(config).await,
            // Self::Transaction(transaction_commands) => transaction_commands.process(config).await,
            Self::Config(config_commands) => config_commands.process(config).await,
        }
    }
}

#[derive(Clone)]
pub struct ActionContext {
    pub config: crate::config::Config,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
    //pub on_before_signing_callback: std::sync::Arc<dyn Fn(&mut near_primitives::transaction::Transaction) -> futures::future::BoxFuture<crate::CliResult>>,
    pub on_before_signing_callback: std::sync::Arc<dyn Fn(&mut near_primitives::transaction::Transaction) -> crate::CliResult>,
    pub on_after_signing_callback: std::sync::Arc<dyn Fn(&near_primitives::transaction::SignedTransaction) -> crate::CliResult>,
}

#[derive(Clone)]
pub struct TransactionContext {
    pub config: crate::config::Config,
    pub network_config: crate::config::NetworkConfig,
    pub transaction: near_primitives::transaction::Transaction,
    pub on_before_signing_callback: std::sync::Arc<dyn Fn(&mut near_primitives::transaction::Transaction) -> crate::CliResult>,
    pub on_after_signing_callback: std::sync::Arc<dyn Fn(&near_primitives::transaction::SignedTransaction) -> crate::CliResult>,
}
