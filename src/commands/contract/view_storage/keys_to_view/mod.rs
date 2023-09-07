use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod all_keys;
mod keys_start_with_bytes_as_base64;
mod keys_start_with_string;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ViewStorageContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select keys to view contract storage state:
pub enum KeysToView {
    #[strum_discriminants(strum(
        message = "all                              - View contract storage state for all keys"
    ))]
    /// View contract storage state for all keys
    All(self::all_keys::AllKeys),
    #[strum_discriminants(strum(
        message = "keys-start-with-string           - View contract storage state for keys that start with a string (for example, \"S\")"
    ))]
    /// View contract storage state for keys that start with a string (for example, "S")
    KeysStartWithString(self::keys_start_with_string::KeysStartWithString),
    #[strum_discriminants(strum(
        message = "keys-start-with-bytes-as-base64  - View contract storage state for keys that start with Base64 bytes (for example, \"Uw==\")"
    ))]
    /// View contract storage state for keys that start with Base64 bytes (for example, "Uw==")
    KeysStartWithBytesAsBase64(self::keys_start_with_bytes_as_base64::KeysStartWithBytesAsBase64),
}

#[derive(Debug, Clone)]
pub struct KeysContext {
    pub global_context: crate::GlobalContext,
    pub contract_account_id: near_primitives::types::AccountId,
    pub prefix: near_primitives::types::StoreKey,
}
