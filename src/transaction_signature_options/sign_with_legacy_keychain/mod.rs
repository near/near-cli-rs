extern crate dirs;

use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};

use crate::common::{RpcResultExt, block_on};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignLegacyKeychainContext)]
pub struct SignLegacyKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    signer_public_key: Option<crate::types::public_key::PublicKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_height: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignLegacyKeychainContext(super::SubmitContext);

impl SignLegacyKeychainContext {
    #[tracing::instrument(
        name = "Signing the transaction with a key saved in legacy keychain ...",
        skip_all
    )]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLegacyKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Signing the transaction with a key saved in legacy keychain ...");

        let network_config = previous_context.network_config.clone();

        let keychain_folder = previous_context
            .global_context
            .config
            .credentials_home_dir
            .join(&network_config.network_name);
        let signer_keychain_folder =
            keychain_folder.join(previous_context.prepopulated_transaction.signer_id.as_str());
        let signer_access_key_file_path: std::path::PathBuf = {
            if previous_context.global_context.offline {
                signer_keychain_folder.join(format!(
                    "{}.json",
                    scope
                        .signer_public_key
                        .as_ref()
                        .wrap_err(
                            "Signer public key is required to sign a transaction in offline mode"
                        )?
                        .to_string()
                        .replace(':', "_")
                ))
            } else if signer_keychain_folder.exists() {
                let full_access_key_filenames = block_on(
                        network_config.client().rpc().view_access_key_list(
                            &previous_context.prepopulated_transaction.signer_id,
                            near_kit::Finality::Final.into(),
                        ),
                    )
                    .into_eyre()
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch access KeyList for {}",
                            previous_context.prepopulated_transaction.signer_id
                        )
                    })?
                    .keys
                    .iter()
                    .filter(|access_key_info| {
                        matches!(
                            access_key_info.access_key.permission,
                            near_kit::AccessKeyPermissionView::FullAccess
                        )
                    })
                    .map(|access_key_info| {
                        format!(
                            "{}.json",
                            access_key_info.public_key.to_string().replace(":", "_")
                        )
                        .into()
                    })
                    .collect::<std::collections::HashSet<std::ffi::OsString>>();

                signer_keychain_folder
                    .read_dir()
                    .wrap_err("There are no access keys found in the keychain for the signer account. Import an access key for an account before signing transactions with keychain.")?
                    .filter_map(Result::ok)
                    .find(|entry| full_access_key_filenames.contains(&entry.file_name()))
                    .map(|signer_access_key| signer_access_key.path())
                    .unwrap_or_else(|| keychain_folder.join(format!(
                        "{}.json",
                        previous_context.prepopulated_transaction.signer_id
                    )))
            } else {
                keychain_folder.join(format!(
                    "{}.json",
                    previous_context.prepopulated_transaction.signer_id
                ))
            }
        };
        let signer_access_key_json =
            std::fs::read(&signer_access_key_file_path).wrap_err_with(|| {
                format!(
                    "Access key file for account <{}> on network <{}> not found! \nSearch location: {:?}",
                    previous_context.prepopulated_transaction.signer_id,
                    network_config.network_name, signer_access_key_file_path
                )
            })?;
        let signer_access_key: super::AccountKeyPair =
            serde_json::from_slice(&signer_access_key_json).wrap_err_with(|| {
                format!(
                    "Error reading data from file: {:?}",
                    &signer_access_key_file_path
                )
            })?;

        let nk_public_key = signer_access_key.public_key.clone();
        let nk_secret_key = signer_access_key.private_key.clone();

        let (nonce, block_hash, block_height) = super::resolve_nonce_and_block(
            &previous_context.network_config,
            &previous_context.prepopulated_transaction.signer_id,
            &nk_public_key,
            previous_context.global_context.offline,
            scope.nonce,
            scope.block_hash,
            scope.block_height,
        )?;

        Ok(Self(super::sign_transaction_with_secret_key(
            nk_public_key,
            nk_secret_key,
            previous_context,
            nonce,
            block_hash,
            block_height,
            scope.meta_transaction_valid_for,
        )?))
    }
}

impl From<SignLegacyKeychainContext> for super::SubmitContext {
    fn from(item: SignLegacyKeychainContext) -> Self {
        item.0
    }
}

impl SignLegacyKeychain {
    fn input_signer_public_key(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key::PublicKey>> {
        if context.global_context.offline {
            let network_config = context.network_config.clone();

            let mut path =
                std::path::PathBuf::from(&context.global_context.config.credentials_home_dir);

            let dir_name = network_config.network_name;
            path.push(&dir_name);

            path.push(context.prepopulated_transaction.signer_id.to_string());

            let signer_dir = path.read_dir()?;

            let key_list = signer_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|file_name_str| file_name_str.starts_with("ed25519_"))
                .map(|file_name_str| file_name_str.replace(".json", "").replace('_', ":"))
                .collect::<Vec<_>>();

            let selected_input = Select::new("Choose a public key:", key_list).prompt()?;

            return Ok(Some(crate::types::public_key::PublicKey::from_str(
                &selected_input,
            )?));
        }
        Ok(None)
    }

    fn input_nonce(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter a nonce for the access key:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_hash(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::crypto_hash::CryptoHash>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<crate::types::crypto_hash::CryptoHash>::new(
                    "Enter recent block hash:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_height(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new(
                    "Enter recent block height:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }
}
