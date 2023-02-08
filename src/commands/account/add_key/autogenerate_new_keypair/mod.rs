use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

// mod print_keypair_to_terminal;
mod save_keypair_to_keychain;
// #[cfg(target_os = "macos")]
// mod save_keypair_to_macos_keychain;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        _scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(crate::common::generate_keypair())?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties,
            public_key,
        })
    }
}

impl interactive_clap::FromCli for GenerateKeypair {
    type FromCliContext = super::access_key_type::AccessTypeContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<GenerateKeypair as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let new_context_scope = InteractiveClapContextScopeForGenerateKeypair {};
        let new_context =
            GenerateKeypairContext::from_previous_context(context.clone(), &new_context_scope)?;

        let optional_save_mode = SaveMode::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.save_mode),
            new_context,
        )?;
        let save_mode = if let Some(save_mode) = optional_save_mode {
            save_mode
        } else {
            return Ok(None);
        };
        Ok(Some(Self { save_mode }))
    }
}

impl GenerateKeypair {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        self.save_mode
            .process(config, prepopulated_unsigned_transaction, permission)
            .await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GenerateKeypairContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Save an access key for this account
pub enum SaveMode {
    // #[cfg(target_os = "macos")]
    // #[strum_discriminants(strum(
    //     message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    // ))]
    // ///Save automatically generated key pair to macOS keychain
    // SaveToMacosKeychain(self::save_keypair_to_macos_keychain::SaveKeypairToMacosKeychain),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    ///Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(self::save_keypair_to_keychain::SaveKeypairToKeychain),
    // #[strum_discriminants(strum(
    //     message = "print-to-terminal        - Print automatically generated key pair in terminal"
    // ))]
    // ///Print automatically generated key pair in terminal
    // PrintToTerminal(self::print_keypair_to_terminal::PrintKeypairToTerminal),
}

impl SaveMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: 0,
            permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self {
            // #[cfg(target_os = "macos")]
            // SaveMode::SaveToMacosKeychain(save_keypair_to_macos_keychain) => {
            //     save_keypair_to_macos_keychain
            //         .process(
            //             config,
            //             key_pair_properties,
            //             prepopulated_unsigned_transaction,
            //         )
            //         .await
            // }
            SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
                save_keypair_to_keychain
                    .process(
                        config,
                        key_pair_properties,
                        prepopulated_unsigned_transaction,
                    )
                    .await
            } // SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
              //     print_keypair_to_terminal
              //         .process(
              //             config,
              //             key_pair_properties,
              //             prepopulated_unsigned_transaction,
              //         )
              //         .await
              // }
        }
    }
}
