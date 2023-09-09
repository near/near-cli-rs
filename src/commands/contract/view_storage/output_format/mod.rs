use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod as_json;
mod as_text;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::keys_to_view::KeysContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a format to view contract storage state:
pub enum OutputFormat {
    #[strum_discriminants(strum(
        message = "as-json    - View contract storage state in JSON format"
    ))]
    /// View contract storage state in JSON format
    AsJson(self::as_json::AsJson),
    #[strum_discriminants(strum(
        message = "as-text    - View contract storage state in the text"
    ))]
    /// View contract storage state in the text
    AsText(self::as_text::AsText),
}
