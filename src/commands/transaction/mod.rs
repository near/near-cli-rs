#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod construct_transaction;
mod send_meta_transaction;
mod send_signed_transaction;
mod sign_transaction;
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
        message = "construct-transaction    - Construct a new transaction"
    ))]
    /// Construct a new transaction
    ConstructTransaction(self::construct_transaction::ConstructTransaction),
    #[strum_discriminants(strum(
        message = "send-meta-transaction    - Act as a relayer to send a signed delegate action (meta-transaction)"
    ))]
    /// Act as a relayer to send a signed delegate action (meta-transaction)
    SendMetaTransaction(self::send_meta_transaction::SendMetaTransaction),
    #[strum_discriminants(strum(
        message = "send-signed-transaction  - Send a signed transaction"
    ))]
    /// Send a signed transaction
    SendSignedTransaction(self::send_signed_transaction::SignedTransaction),
    #[strum_discriminants(strum(
        message = "sign-transaction         - Sign base64 encoding transaction"
    ))]
    /// Sign base64 encoding transaction
    SignTransaction(self::sign_transaction::SignTransaction),
}
