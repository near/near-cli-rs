use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod base64_args;
mod file_args;
mod json_args;
mod manually;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::UpdateSocialProfileContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to pass profile arguments?
pub enum ProfileArgsType {
    #[strum_discriminants(strum(message = "manually     - Interactive input of arguments"))]
    /// Interactive input of arguments
    Manually(self::manually::Manually),
    #[strum_discriminants(strum(
        message = "json-args    - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    JsonArgs(self::json_args::JsonArgs),
    #[strum_discriminants(strum(message = "base64-args  - Base64-encoded string (e.g. e30=)"))]
    /// Base64-encoded string (e.g. e30=)
    Base64Args(self::base64_args::Base64Args),
    #[strum_discriminants(strum(message = "file-args    - Read from JSON file"))]
    /// Read from JSON file
    FileArgs(self::file_args::FileArgs),
}

#[derive(Clone)]
pub struct ArgsContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
    pub data: Vec<u8>,
}
