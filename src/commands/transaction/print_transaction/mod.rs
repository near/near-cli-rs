#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod signed;
mod unsigned;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct PrintTransactionCommands {
    #[interactive_clap(subcommand)]
    show_transaction_actions: PrintTransactionActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select signed or unsigned transaction to print:
pub enum PrintTransactionActions {
    #[strum_discriminants(strum(
        message = "unsigned         - Print all fields of previously prepared unsigned transaction without modification"
    ))]
    /// Print previously prepared unsigned transaction without modification
    Unsigned(self::unsigned::PrintTransaction),
    #[strum_discriminants(strum(
        message = "signed           - Print all fields of previously prepared signed transaction without modification"
    ))]
    /// Send a signed transaction
    Signed(self::signed::PrintTransaction),
}
