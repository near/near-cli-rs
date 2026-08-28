use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypair {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Which signature scheme should the new key pair use?
    signature_scheme: crate::common::SignatureScheme,
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Clone)]
pub struct GenerateKeypairContext {
    config: crate::config::Config,
    new_account_id: crate::types::account_id::AccountId,
    public_key: near_kit::PublicKey,
    generated_key_pair: crate::common::GeneratedKeyPair,
    on_before_creating_account_callback: super::super::network::OnBeforeCreatingAccountCallback,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let generated_key_pair =
            crate::common::GeneratedKeyPair::generate(&scope.signature_scheme)?;
        let public_key = generated_key_pair.public_key()?;

        Ok(Self {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key,
            generated_key_pair,
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        })
    }
}

impl GenerateKeypair {
    fn input_signature_scheme(
        _context: &super::super::NewAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::SignatureScheme>> {
        crate::common::input_signature_scheme()
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = SaveModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Save an access key for this account:
pub enum SaveMode {
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to keychain"
    ))]
    /// Save automatically generated key pair to keychain
    SaveToKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "save-to-legacy-keychain  - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToLegacyKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(SaveKeyPair),
}

#[derive(Clone)]
pub struct SaveModeContext(super::super::SponsorServiceContext);

impl SaveModeContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let scope = *scope;

        let on_after_getting_network_callback: super::super::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let new_account_id_str = previous_context.new_account_id.to_string();
                let generated_key_pair = previous_context.generated_key_pair.clone();
                let credentials_home_dir = previous_context.config.credentials_home_dir.clone();

                move |network_config| {
                    match scope {
                        SaveModeDiscriminants::SaveToKeychain => {
                            crate::common::save_access_key_to_keychain_or_save_to_legacy_keychain(
                                network_config.clone(),
                                credentials_home_dir.clone(),
                                &generated_key_pair.keychain_json()?,
                                &generated_key_pair.keychain_key_id()?,
                                &new_account_id_str,
                            )
                        }
                        SaveModeDiscriminants::SaveToLegacyKeychain => {
                            crate::common::save_access_key_to_legacy_keychain(
                                network_config.clone(),
                                credentials_home_dir.clone(),
                                &generated_key_pair.keychain_json()?,
                                &generated_key_pair.keychain_key_id()?,
                                &new_account_id_str,
                            )
                        }
                        SaveModeDiscriminants::PrintToTerminal => {
                            Ok(generated_key_pair.terminal_info())
                        }
                    }
                }
            });

        Ok(Self(super::super::SponsorServiceContext {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key: previous_context.public_key,
            on_after_getting_network_callback,
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        }))
    }
}

impl From<SaveModeContext> for super::super::SponsorServiceContext {
    fn from(item: SaveModeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::SponsorServiceContext)]
pub struct SaveKeyPair {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::super::network::Network,
}
