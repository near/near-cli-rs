use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use near_primitives::account::id::AccountType;

pub mod key_store_prop;
mod network;
mod using_private_key;
mod using_seed_phrase;
mod using_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ImportAccountCommandContext)]
pub struct ImportAccountCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID do you want to import?
    account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subcommand)]
    /// How would you like to import the account?
    import_account_actions: ImportAccountActions,
}

#[derive(Debug, Clone)]
pub struct ImportAccountCommandContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl ImportAccountCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ImportAccountCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl ImportAccountCommand {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID do you want to import?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ImportAccountCommandContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to import the account?
pub enum ImportAccountActions {
    #[strum_discriminants(strum(
        message = "using-web-wallet          - Import existing account using NEAR Wallet (a.k.a. \"sign in\")"
    ))]
    /// Import existing account using NEAR Wallet (a.k.a. "sign in")
    UsingWebWallet(self::using_web_wallet::LoginFromWebWallet),
    #[strum_discriminants(strum(
        message = "using-seed-phrase         - Import existing account using a seed phrase"
    ))]
    /// Import existing account using a seed phrase
    UsingSeedPhrase(self::using_seed_phrase::LoginFromSeedPhrase),
    #[strum_discriminants(strum(
        message = "using-private-key         - Import existing account using a private key"
    ))]
    /// Import existing account using a private key
    UsingPrivateKey(self::using_private_key::LoginFromPrivateKey),
}

pub fn check_account_id(
    global_context: &crate::GlobalContext,
    chosen_network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    public_key: &near_crypto::PublicKey,
) -> crate::CliResult {
    // Implicit account always exists - we import it without prompting to check if it exists.
    if let AccountType::NearImplicitAccount = account_id.get_account_type() {
        return Ok(());
    }

    if !crate::common::is_account_exist(global_context, account_id.clone())? {
        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
            "Couldn't find account <{account_id}> on any known network"
        ));
    }

    let access_key_view = crate::common::verify_account_access_key(
        account_id.clone(),
        public_key.clone(),
        chosen_network_config.clone(),
    );
    if let Err(err @ crate::common::AccountStateError::Cancel) = access_key_view {
        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err));
    }
    if access_key_view.is_err() {
        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
            "Couldn't find access key for account <{account_id}> on network <{}>:\n    access_key: {public_key}",
            chosen_network_config.network_name
        ));
    }

    Ok(())
}
