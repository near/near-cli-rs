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

pub type OnAfterGettingNetworkConfigCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig) -> color_eyre::eyre::Result<()>>;

#[derive(Clone)]
pub struct NetworkForImportAccountContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
    pub key_store_property: super::key_store_prop::KeyStorePropertyType,
    pub on_after_getting_network_callback: OnAfterGettingNetworkConfigCallback,
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

        (previous_context.on_after_getting_network_callback)(&network_config)?;

        if scope.check_account_id {
            super::check_account_id(
                &previous_context.global_context,
                &network_config,
                &previous_context.account_id,
                previous_context.key_store_property.public_key(),
            )?;
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
        use near_primitives::account::id::AccountType;

        // Implicit accounts always exist - nothing to verify
        if let AccountType::NearImplicitAccount = context.account_id.get_account_type() {
            return Ok(false);
        }

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
