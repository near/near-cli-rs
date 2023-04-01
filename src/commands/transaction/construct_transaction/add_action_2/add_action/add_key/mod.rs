use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod access_key_type;
mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::ConstructTransactionContext)]
pub struct AddKeyAction {
    #[interactive_clap(subcommand)]
    permission: AccessKeyPermission,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a permission that you want to add to the access key
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(
        message = "grant-full-access           - A permission with full access"
    ))]
    /// Provide data for a full access key
    GrantFullAccess(self::access_key_type::FullAccessType),
    #[strum_discriminants(strum(
        message = "grant-function-call-access  - A permission with function call"
    ))]
    /// Provide data for a function-call access key
    GrantFunctionCallAccess(self::access_key_type::FunctionCallType),
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = self::access_key_type::AccessKeyPermissionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Add an access key for this account
pub enum AccessKeyMode {
    #[strum_discriminants(strum(
        message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    ))]
    /// Use the provided seed phrase manually
    UseManuallyProvidedSeedPhrase(
        self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    /// Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
}
