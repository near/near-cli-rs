use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod print_keypair_to_terminal;
mod save_keypair_to_keychain;
mod save_keypair_to_legacy_keychain;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(skip_default_from_cli)]
pub struct NetworkForImportAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the name of the network?
    network_name: String,

    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    wallet_url: Option<crate::types::url::Url>,

    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Check that the account exists on the network and holds this key?
    check_account_id: bool,

    #[interactive_clap(subcommand)]
    /// How do you like to save imported account?
    save_mode: SaveMode,
}

/// Called once the network is chosen. The `bool` is whether the account/key check
/// (`--check-account-id`) will run next.
pub type OnAfterGettingNetworkConfigCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig, bool) -> color_eyre::eyre::Result<()>>;

#[derive(Clone)]
pub struct NetworkForImportAccountContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
    pub key_store_property: super::key_store_prop::KeyStorePropertyType,
    pub on_after_getting_network_callback: OnAfterGettingNetworkConfigCallback,
    pub uses_web_wallet: bool,
}

#[derive(Clone)]
pub struct NetworkForImportAccountOutputContext {
    pub config: crate::config::Config,
    pub chosen_network_config: crate::config::NetworkConfig,
    pub account_id: near_primitives::types::AccountId,
    pub key_store_property: super::key_store_prop::KeyStorePropertyType,
}

impl NetworkForImportAccountOutputContext {
    pub fn from_previous_context(
        previous_context: NetworkForImportAccountContext,
        scope: &<NetworkForImportAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_connection = previous_context
            .global_context
            .config
            .network_connection
            .clone();
        let mut network_config = network_connection
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        if let Some(url) = scope.wallet_url.clone() {
            network_config.wallet_url = url.into();
        }

        (previous_context.on_after_getting_network_callback)(
            &network_config,
            scope.check_account_id,
        )?;

        if scope.check_account_id {
            super::check_account_id(
                &previous_context.global_context,
                &network_config,
                &previous_context.account_id,
                previous_context.key_store_property.public_key(),
            )?;
        } else {
            super::warn_on_implicit_account_id_missmatch(
                &previous_context.account_id,
                previous_context.key_store_property.public_key(),
            );
        }

        Ok(Self {
            config: previous_context.global_context.config,
            chosen_network_config: network_config,
            account_id: previous_context.account_id,
            key_store_property: previous_context.key_store_property,
        })
    }
}

impl NetworkForImportAccount {
    fn input_check_account_id(
        context: &NetworkForImportAccountContext,
    ) -> color_eyre::eyre::Result<bool> {
        use inquire::Select;

        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to check.")]
            Yes,
            #[strum(to_string = "No, I just want to save the access key.")]
            No,
        }
        let select_choose_input = Select::new(
            format!(
                "Would you like to check if account <{}> has the access key?",
                context.account_id
            )
            .as_str(),
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;

        Ok(matches!(select_choose_input, ConfirmOptions::Yes))
    }

    /// Lets the user pick which web wallet approves the new key, when `--wallet-url` wasn't passed.
    /// Returns `None` (keep the network's configured `wallet_url`) outside an interactive terminal
    /// or when there is nothing to choose from.
    fn input_wallet_url(
        context: &NetworkForImportAccountContext,
        network_connection_name: &str,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        use std::io::IsTerminal;

        if !std::io::stdin().is_terminal() {
            return Ok(None);
        }

        let Some(network_config) = context
            .global_context
            .config
            .network_connection
            .get(network_connection_name)
        else {
            return Ok(None);
        };
        let configured_url = network_config.wallet_url.clone();

        let mut choices: Vec<WalletChoice> = known_web_wallets(&network_config.network_name)
            .into_iter()
            .map(|(name, url)| WalletChoice::Known { name, url })
            .collect();
        if !choices.iter().any(
            |choice| matches!(choice, WalletChoice::Known { url, .. } if *url == configured_url),
        ) {
            choices.insert(0, WalletChoice::Configured(configured_url.clone()));
        }
        choices.push(WalletChoice::Custom);

        let starting_cursor = choices
            .iter()
            .position(|choice| choice.url() == Some(&configured_url))
            .unwrap_or(0);

        let selected = inquire::Select::new(
            "Which wallet do you want to use to approve the new access key?",
            choices,
        )
        .with_starting_cursor(starting_cursor)
        .prompt()?;

        let url = match selected {
            WalletChoice::Known { url, .. } | WalletChoice::Configured(url) => url,
            WalletChoice::Custom => {
                let crate::types::url::Url(mut url) =
                    inquire::CustomType::<crate::types::url::Url>::new("Enter the wallet URL:")
                        .prompt()?;
                // The login page is resolved relative to this URL, so it must end with `/`.
                if !url.path().ends_with('/') {
                    let path = format!("{}/", url.path());
                    url.set_path(&path);
                }
                url
            }
        };

        Ok(Some(url.into()))
    }

    fn input_network_name(
        context: &NetworkForImportAccountContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(
            &context.global_context.config,
            std::slice::from_ref(&context.account_id),
        )
    }
}

#[derive(Clone, Debug, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkForImportAccountOutputContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How to save an imported account:
pub enum SaveMode {
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to keychain"
    ))]
    /// Save automatically generated key pair to keychain
    SaveToKeychain(save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "save-to-legacy-keychain  - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToLegacyKeychain(save_keypair_to_legacy_keychain::SaveKeypairToLegacyKeychain),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(print_keypair_to_terminal::PrintKeypairToTerminal),
}

impl interactive_clap::FromCli for NetworkForImportAccount {
    type FromCliContext = NetworkForImportAccountContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<
            <NetworkForImportAccount as interactive_clap::ToCli>::CliVariant,
        >,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        // Was the rest of this step spelled out on the command line?
        let fully_scripted =
            clap_variant.network_name.is_some() && clap_variant.save_mode.is_some();

        if clap_variant.network_name.is_none() {
            clap_variant.network_name = match Self::input_network_name(&context) {
                Ok(Some(name)) => Some(name),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let network_name = clap_variant.network_name.clone().expect("Unexpected error");

        if clap_variant.wallet_url.is_none() && context.uses_web_wallet && !fully_scripted {
            match Self::input_wallet_url(&context, &network_name) {
                Ok(wallet_url) => clap_variant.wallet_url = wallet_url,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            }
        }

        if !clap_variant.check_account_id && !fully_scripted {
            clap_variant.check_account_id = match Self::input_check_account_id(&context) {
                Ok(answer) => answer,
                Err(err) => {
                    return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
                }
            }
        }

        let new_context_scope = InteractiveClapContextScopeForNetworkForImportAccount {
            wallet_url: clap_variant.wallet_url.clone(),
            check_account_id: clap_variant.check_account_id,
            network_name,
        };
        let new_context = match NetworkForImportAccountOutputContext::from_previous_context(
            context,
            &new_context_scope,
        ) {
            Ok(new_context) => new_context,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };

        match <SaveMode as interactive_clap::FromCli>::from_cli(
            clap_variant.save_mode.take(),
            new_context,
        ) {
            interactive_clap::ResultFromCli::Ok(cli_save_mode)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_save_mode)) => {
                clap_variant.save_mode = Some(cli_save_mode);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(_) => {
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional, err) => {
                clap_variant.save_mode = optional;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

/// Wallets known to support the CLI web-wallet login (`<wallet_url>login/?public_key=...`), listed
/// alphabetically. Each URL is the base the CLI appends `login/` to.
fn known_web_wallets(network_name: &str) -> Vec<(&'static str, url::Url)> {
    if !matches!(network_name, "mainnet" | "testnet") {
        return Vec::new();
    }

    [
        // Intear uses the network of the account selected in the wallet.
        ("Intear Wallet", "https://wallet.intear.tech/".to_string()),
        (
            "Meteor Wallet",
            format!("https://wallet.meteorwallet.app/cli/{network_name}/"),
        ),
    ]
    .into_iter()
    .filter_map(|(name, url)| Some((name, url.parse().ok()?)))
    .collect()
}

#[derive(Clone)]
enum WalletChoice {
    Known { name: &'static str, url: url::Url },
    Configured(url::Url),
    Custom,
}

impl WalletChoice {
    fn url(&self) -> Option<&url::Url> {
        match self {
            Self::Known { url, .. } | Self::Configured(url) => Some(url),
            Self::Custom => None,
        }
    }
}

impl std::fmt::Display for WalletChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known { name, url } => write!(f, "{name} ({url})"),
            Self::Configured(url) => write!(f, "Wallet from your config ({url})"),
            Self::Custom => write!(f, "Enter a custom wallet URL"),
        }
    }
}
