use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod key_store_prop;
mod network;
mod using_private_key;
mod using_seed_phrase;
mod using_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImportAccountCommand {
    #[interactive_clap(subcommand)]
    /// How would you like to import the account?
    import_account_actions: ImportAccountActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
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

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ImportAccountDetailsContext)]
#[interactive_clap(output_context = network::NetworkForImportAccountContext)]
pub struct ImportAccountDetails {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID do you want to import?
    account_id: crate::types::account_id::AccountId,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: network::NetworkForImportAccount,
}

#[derive(Clone)]
pub struct ImportAccountDetailsContext {
    global_context: crate::GlobalContext,
    key_store_property: key_store_prop::KeyStorePropertyType,
    on_after_getting_network_callback: network::OnAfterGettingNetworkConfigCallback,
}

impl network::NetworkForImportAccountContext {
    pub fn from_previous_context(
        previous_context: ImportAccountDetailsContext,
        scope: &<ImportAccountDetails as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

        Ok(Self {
            global_context: previous_context.global_context,
            account_id,
            key_store_property: previous_context.key_store_property,
            on_after_getting_network_callback: previous_context.on_after_getting_network_callback,
        })
    }
}

impl ImportAccountDetails {
    pub fn input_account_id(
        context: &ImportAccountDetailsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What Account ID do you want to import?",
        )
    }
}

fn warn_on_implicit_account_id_missmatch(
    account_id: &near_primitives::types::AccountId,
    public_key: &near_crypto::PublicKey,
) {
    use color_eyre::owo_colors::OwoColorize;

    let Ok(derived_account_id) = near_crypto::PublicKey::from_near_implicit_account(account_id)
    else {
        return;
    };

    if derived_account_id == *public_key {
        return;
    }

    let info_str = format!(
        "{}\n{}",
        format!(
            "<{account_id}> is a NEAR implicit account id, but it is not derived from <{public_key}>."
        ).yellow(),
        "This key will only be able to sign if it was added to the account on chain. Re-run with --check-account-id to verify it.".yellow()
    );
    tracing::warn!(
        parent: &tracing::Span::none(),
        "\n{}",
        crate::common::indent_payload(&info_str)
    );
}

fn check_account_id(
    global_context: &crate::GlobalContext,
    chosen_network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    public_key: &near_crypto::PublicKey,
) -> crate::CliResult {
    if !crate::common::is_account_exist(global_context, account_id.clone())? {
        // Implicit AccountId always exists. If the public key that was passed to this function is
        // the same public key that was used to generate implicit AccountId, then we don't need
        // to return error as implicit AccountId will be instantiated on the first transaction.
        if near_crypto::PublicKey::from_near_implicit_account(account_id)
            .is_ok_and(|derived_public_key| derived_public_key == *public_key)
        {
            return Ok(());
        }
        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
            "Couldn't find account <{account_id}> on any known network"
        ));
    }

    match crate::common::verify_account_access_key(
        account_id.clone(),
        public_key.clone(),
        chosen_network_config.clone(),
    ) {
        Err(err @ crate::common::AccountStateError::Cancel) => Err(color_eyre::eyre::eyre!(err)),
        Err(_) => Err(color_eyre::eyre::eyre!(
            "Couldn't find access key for account <{account_id}> on network <{}>:\n    access_key: {public_key}",
            chosen_network_config.network_name
        )),
        _ => Ok(()),
    }
}
