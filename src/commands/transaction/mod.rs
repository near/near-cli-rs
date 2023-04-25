#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod construct_transaction;
mod delegate_action;
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
/// Сhoose action for transaction
pub enum TransactionActions {
    #[strum_discriminants(strum(message = "view-status            - View a transaction status"))]
    /// Execute function (contract method)
    ViewStatus(self::view_status::TransactionInfo),
    #[strum_discriminants(strum(
        message = "construct-transaction  - Construct a new transaction"
    ))]
    /// Construct a new transaction
    ConstructTransaction(self::construct_transaction::ConstructTransaction),
    #[strum_discriminants(strum(
        message = "send-delegate-transaction  - Sending by the relayer a signed delegate action"
    ))]
    /// Sending by the relayer a signed delegate action
    SendDelegateAction(self::delegate_action::TransactionInfo),
}
