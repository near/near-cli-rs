use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod all_keys;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ViewStorageContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select keys to view contract storage:
pub enum KeysToView {
    #[strum_discriminants(strum(
        message = "all                              - View contract storage for all keys"
    ))]
    /// View contract storage for all keys
    All(self::all_keys::AllKeys),
    #[strum_discriminants(strum(
        message = "keys-start-with-string           - View contract storage for keys that start with a string (for example, \"S\")"
    ))]
    /// View contract storage for keys that start with a string (for example, "S")
    KeysStartWithString,
    #[strum_discriminants(strum(
        message = "keys-start-with-bytes-as-base64  - View the contract storage for keys that start with Base64 bytes (for example, \"Uw==\")"
    ))]
    /// View the contract storage for keys that start with Base64 bytes (for example, "Uw==")
    KeysStartWithBytesAsBase64,
}

#[derive(Debug, Clone)]
pub struct KeysContext {
    pub global_context: crate::GlobalContext,
    pub contract_account_id: near_primitives::types::AccountId,
    pub prefix: near_primitives::types::StoreKey,
}
