#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod construct_transaction;
mod print_transaction;
mod reconstruct_transaction;
mod send_meta_transaction;
pub mod send_signed_transaction;
pub mod sign_transaction;
mod view_status;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TransactionCommands {
    #[interactive_clap(subcommand)]
    transaction_actions: TransactionActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Сhoose action for transaction:
pub enum TransactionActions {
    #[strum_discriminants(strum(
        message = "view-status              - View a transaction status"
    ))]
    /// Execute function (contract method)
    ViewStatus(self::view_status::TransactionInfo),
    #[strum_discriminants(strum(
        message = "reconstruct-transaction  - Use any existing transaction from the chain to construct NEAR CLI command (helpful tool for re-submitting similar transactions)"
    ))]
    /// Use any existing transaction from the chain to construct NEAR CLI command (helpful tool for re-submitting similar transactions)
    ReconstructTransaction(self::reconstruct_transaction::TransactionInfo),
    #[strum_discriminants(strum(
        message = "construct-transaction    - Construct a new transaction"
    ))]
    /// Construct a new transaction
    ConstructTransaction(self::construct_transaction::ConstructTransaction),
    #[strum_discriminants(strum(
        message = "sign-transaction         - Sign previously prepared unsigned transaction"
    ))]
    /// Sign previously prepared unsigned transaction
    SignTransaction(self::sign_transaction::SignTransaction),
    #[strum_discriminants(strum(
        message = "print-transaction        - Print all fields of previously prepared transaction without modification"
    ))]
    /// Print previously prepared unsigned transaction without modification
    PrintTransaction(self::print_transaction::PrintTransactionCommands),
    #[strum_discriminants(strum(
        message = "send-signed-transaction  - Send a signed transaction"
    ))]
    /// Send a signed transaction
    SendSignedTransaction(self::send_signed_transaction::SignedTransaction),
    #[strum_discriminants(strum(
        message = "send-meta-transaction    - Act as a relayer to send a signed delegate action (meta-transaction)"
    ))]
    /// Act as a relayer to send a signed delegate action (meta-transaction)
    SendMetaTransaction(self::send_meta_transaction::SendMetaTransaction),
}
