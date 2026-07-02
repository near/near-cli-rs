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

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    global_context: crate::GlobalContext,
    account_properties: super::super::AccountProperties,
    generated_key_pair: crate::common::GeneratedKeyPair,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let generated_key_pair =
            crate::common::GeneratedKeyPair::generate(&scope.signature_scheme)?;
        let public_key = generated_key_pair.public_key()?;
        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self {
            global_context: previous_context.global_context,
            account_properties,
            generated_key_pair,
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

impl From<GenerateKeypairContext> for super::super::AccountPropertiesContext {
    fn from(item: GenerateKeypairContext) -> Self {
        Self {
            global_context: item.global_context,
            account_properties: item.account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
        }
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
    SaveToKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "save-to-legacy-keychain  - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToLegacyKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(SignAs),
}

#[derive(Clone)]
pub struct SaveModeContext(super::super::AccountPropertiesContext);

impl SaveModeContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let scope = *scope;

        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new({
                let new_account_id = previous_context.account_properties.new_account_id.clone();
                let generated_key_pair = previous_context.generated_key_pair.clone();
                let credentials_home_dir = previous_context.global_context.config.credentials_home_dir.clone();

                move |_transaction, network_config| {
                    match scope {
                        SaveModeDiscriminants::SaveToKeychain => {
                            crate::common::save_access_key_to_keychain_or_save_to_legacy_keychain(
                                network_config.clone(),
                                credentials_home_dir.clone(),
                                &generated_key_pair.keychain_json()?,
                                &generated_key_pair.keychain_key_id()?,
                                new_account_id.as_ref(),
                            )
                        }
                        SaveModeDiscriminants::SaveToLegacyKeychain => {
                            crate::common::save_access_key_to_legacy_keychain(
                                network_config.clone(),
                                credentials_home_dir.clone(),
                                &generated_key_pair.keychain_json()?,
                                &generated_key_pair.keychain_key_id()?,
                                new_account_id.as_ref(),
                            )
                        }
                        SaveModeDiscriminants::PrintToTerminal => {
                            Ok(generated_key_pair.terminal_info())
                        }
                    }
                }
            });

        Ok(Self(super::super::AccountPropertiesContext {
            global_context: previous_context.global_context,
            account_properties: previous_context.account_properties,
            on_before_sending_transaction_callback,
        }))
    }
}

impl From<SaveModeContext> for super::super::AccountPropertiesContext {
    fn from(item: SaveModeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::AccountPropertiesContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}
