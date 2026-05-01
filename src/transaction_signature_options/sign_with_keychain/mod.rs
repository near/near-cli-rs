use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::CustomType;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::{RpcResultExt, block_on};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignKeychainContext)]
pub struct SignKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    signer_public_key: Option<crate::types::public_key::PublicKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    block_height: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignKeychainContext(super::SubmitContext);

impl From<super::sign_with_legacy_keychain::SignLegacyKeychainContext> for SignKeychainContext {
    fn from(value: super::sign_with_legacy_keychain::SignLegacyKeychainContext) -> Self {
        SignKeychainContext(value.into())
    }
}

impl SignKeychainContext {
    #[tracing::instrument(
        name = "Signing the transaction with a key saved in the secure keychain ...",
        skip_all
    )]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Signing the transaction with a key saved in the secure keychain ...");

        let network_config = previous_context.network_config.clone();

        let service_name = std::borrow::Cow::Owned(format!(
            "near-{}-{}",
            network_config.network_name,
            previous_context.prepopulated_transaction.signer_id.as_str()
        ));

        let password = if previous_context.global_context.offline {
            let res = keyring::Entry::new(
                &service_name,
                &format!(
                    "{}:{}",
                    previous_context.prepopulated_transaction.signer_id,
                    scope.signer_public_key.clone().wrap_err(
                        "Signer public key is required to sign a transaction in offline mode"
                    )?
                ),
            )?
            .get_password();

            match res {
                Ok(password) => password,
                Err(err) => {
                    match matches!(err, keyring::Error::NoEntry) {
                        true => eprintln!("Warning: no access key found in keychain"),
                        false => eprintln!("Warning: keychain was not able to be read, {err}"),
                    }

                    eprintln!("trying with the legacy keychain");
                    return from_legacy_keychain(previous_context, scope);
                }
            }
        } else {
            let access_key_list = block_on(network_config.client().rpc().view_access_key_list(
                &previous_context.prepopulated_transaction.signer_id,
                near_kit::Finality::Final.into(),
            ))
            .into_eyre()
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch access key list for {}",
                    previous_context.prepopulated_transaction.signer_id
                )
            })?;

            let res = access_key_list
                .keys
                .into_iter()
                .filter(|key| {
                    matches!(
                        key.access_key.permission,
                        near_kit::AccessKeyPermissionView::FullAccess
                    )
                })
                .map(|key| key.public_key)
                .find_map(|public_key| {
                    let keyring = keyring::Entry::new(
                        &service_name,
                        &format!(
                            "{}:{}",
                            previous_context.prepopulated_transaction.signer_id, public_key
                        ),
                    )
                    .ok()?;
                    keyring.get_password().ok()
                });

            match res {
                Some(password) => password,
                None => {
                    // no access keys found, try the legacy keychain
                    warning_message("no access keys found in keychain, trying legacy keychain");
                    return from_legacy_keychain(previous_context, scope);
                }
            }
        };

        let account_json: super::AccountKeyPair =
            serde_json::from_str(&password).wrap_err("Error reading data")?;

        let nk_public_key = account_json.public_key.clone();
        let nk_secret_key = account_json.private_key.clone();

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

#[tracing::instrument(name = "Warning:", skip_all)]
fn warning_message(instrument_message: &str) {
    tracing::Span::current().pb_set_message(instrument_message);
    tracing::warn!(target: "near_teach_me", "{instrument_message}");
    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[tracing::instrument(name = "Trying to sign with the legacy keychain ...", skip_all)]
fn from_legacy_keychain(
    previous_context: crate::commands::TransactionContext,
    scope:  &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
) -> color_eyre::eyre::Result<SignKeychainContext> {
    tracing::info!(target: "near_teach_me", "Trying to sign with the legacy keychain ...");
    let legacy_scope =
        super::sign_with_legacy_keychain::InteractiveClapContextScopeForSignLegacyKeychain {
            signer_public_key: scope.signer_public_key.clone(),
            nonce: scope.nonce,
            block_hash: scope.block_hash,
            block_height: scope.block_height,
            meta_transaction_valid_for: scope.meta_transaction_valid_for,
        };

    Ok(
        super::sign_with_legacy_keychain::SignLegacyKeychainContext::from_previous_context(
            previous_context,
            &legacy_scope,
        )?
        .into(),
    )
}

impl From<SignKeychainContext> for super::SubmitContext {
    fn from(item: SignKeychainContext) -> Self {
        item.0
    }
}

impl SignKeychain {
    fn input_signer_public_key(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key::PublicKey>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<crate::types::public_key::PublicKey>::new("Enter public_key:")
                    .prompt()?,
            ));
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
                CustomType::<u64>::new("Enter recent block height:").prompt()?,
            ));
        }
        Ok(None)
    }
}
