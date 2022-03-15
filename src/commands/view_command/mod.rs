use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view_account;
mod view_contract_code;
mod view_contract_state;
mod view_nonce;
mod view_recent_block_hash;
mod view_transaction_status;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct ViewQueryRequest {
    #[interactive_clap(subcommand)]
    pub query: QueryRequest,
}

impl ViewQueryRequest {
    pub async fn process(self) -> crate::CliResult {
        self.query.process().await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///Choose what you want to view
pub enum QueryRequest {
    #[strum_discriminants(strum(message = "View properties for an account"))]
    /// View properties for an account
    AccountSummary(self::view_account::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a contract code"))]
    /// View a contract code
    ContractCode(self::view_contract_code::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a contract state"))]
    /// View a contract state
    ContractState(self::view_contract_state::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a transaction status"))]
    /// View a transaction status
    Transaction(self::view_transaction_status::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a nonce for a public key"))]
    /// View a nonce for a public key
    Nonce(self::view_nonce::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View recent block hash for this network"))]
    /// View recent block hash for this network
    RecentBlockHash(self::view_recent_block_hash::operation_mode::OperationMode),
}

impl QueryRequest {
    pub async fn process(self) -> crate::CliResult {
        match self {
            QueryRequest::AccountSummary(operation_mode) => operation_mode.process().await,
            QueryRequest::ContractCode(operation_mode) => operation_mode.process().await,
            QueryRequest::ContractState(operation_mode) => operation_mode.process().await,
            QueryRequest::Transaction(operation_mode) => operation_mode.process().await,
            QueryRequest::Nonce(operation_mode) => operation_mode.process().await,
            QueryRequest::RecentBlockHash(operation_mode) => operation_mode.process().await,
        }
    }
}
