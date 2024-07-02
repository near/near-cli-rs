#![allow(clippy::arc_with_non_send_sync)]
pub use common::CliResult;

pub mod commands;
pub mod common;
pub mod config;
pub mod js_command_match;
pub mod network;
pub mod network_for_transaction;
pub mod network_view_at_block;
pub mod transaction_signature_options;
pub mod types;
pub mod utils_command;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub config: crate::config::Config,
    pub offline: bool,
    pub teach_me: bool,
}
