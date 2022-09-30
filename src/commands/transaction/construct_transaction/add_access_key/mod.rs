use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod access_key_type;
mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AddKeyCommand {
    #[interactive_clap(subcommand)]
    permission: AccessKeyPermission,
}

impl AddKeyCommand {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self.permission.clone() {
            AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            AccessKeyPermission::GrantFunctionCallAccess(function_call_type) => {
                function_call_type
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select a permission that you want to add to the access key
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
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Add an access key for this account
pub enum AccessKeyMode {
    #[strum_discriminants(strum(
        message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    ))]
    ///Use the provided seed phrase manually
    UseManuallyProvidedSeedPhrase(
        self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    ///Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
}

impl AccessKeyMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        match self {
            AccessKeyMode::UseManuallyProvidedPublicKey(add_access_key_action) => {
                add_access_key_action
                    .process(config, prepopulated_unsigned_transaction, permission)
                    .await
            }
            AccessKeyMode::UseManuallyProvidedSeedPhrase(add_access_with_seed_phrase_action) => {
                add_access_with_seed_phrase_action
                    .process(config, prepopulated_unsigned_transaction, permission)
                    .await
            }
        }
    }
}
