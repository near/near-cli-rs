#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod account;
mod config;
mod contract;
mod staking;
mod tokens;
mod transaction;

#[cfg(feature = "self-update")]
pub mod extensions;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
#[non_exhaustive]
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
    #[strum_discriminants(strum(
        message = "staking     - Manage staking: view, add and withdraw stake"
    ))]
    /// Use this for manage staking: view, add and withdraw stake
    Staking(self::staking::Staking),
    #[strum_discriminants(strum(
        message = "contract    - Manage smart-contracts: deploy code, call functions"
    ))]
    /// Use this for contract actions: call function, deploy, download wasm, inspect storage
    Contract(self::contract::ContractCommands),
    #[strum_discriminants(strum(message = "transaction - Operate transactions"))]
    /// Use this to construct transactions or view a transaction status.
    Transaction(self::transaction::TransactionCommands),
    #[strum_discriminants(strum(
        message = "config      - Manage connections in a configuration file (config.toml)"
    ))]
    /// Use this to manage connections in a configuration file (config.toml).
    Config(self::config::ConfigCommands),
    #[cfg(feature = "self-update")]
    #[strum_discriminants(strum(message = "extension   - Manage near CLI and extensions"))]
    /// Use this to manage near CLI and extensions
    Extensions(self::extensions::ExtensionsCommands),
}

pub type OnBeforeSigningCallback = std::sync::Arc<
    dyn Fn(
        &mut near_primitives::transaction::Transaction,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;

pub type OnAfterGettingNetworkCallback = std::sync::Arc<
    dyn Fn(&crate::config::NetworkConfig) -> color_eyre::eyre::Result<PrepopulatedTransaction>,
>;

#[derive(Debug, Clone)]
pub struct PrepopulatedTransaction {
    pub signer_id: near_primitives::types::AccountId,
    pub receiver_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
}

#[derive(Clone)]
pub struct ActionContext {
    pub global_context: crate::GlobalContext,
    pub interacting_with_account_ids: Vec<near_primitives::types::AccountId>,
    pub on_after_getting_network_callback: OnAfterGettingNetworkCallback,
    pub on_before_signing_callback: OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

#[derive(Clone)]
pub struct TransactionContext {
    pub global_context: crate::GlobalContext,
    pub network_config: crate::config::NetworkConfig,
    pub prepopulated_transaction: PrepopulatedTransaction,
    pub on_before_signing_callback: OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}
