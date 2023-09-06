use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod as_json;
mod as_table;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::keys_to_view::KeysContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a format to view contract storage:
pub enum OutputFormat {
    #[strum_discriminants(strum(message = "as-json      - View contract storage in JSON format"))]
    /// View contract storage in JSON format
    AsJson(self::as_json::AsJson),
    #[strum_discriminants(strum(message = "as-table     - View contract storage in the table"))]
    /// View contract storage in the table
    AsTable(self::as_table::AsTable),
}
