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
pub struct SignKeychainContext {
    network_config: crate::config::NetworkConfig,
    global_context: crate::GlobalContext,
    signed_transaction_or_signed_delegate_action: super::SignedTransactionOrSignedDelegateAction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl From<super::sign_with_legacy_keychain::SignLegacyKeychainContext> for SignKeychainContext {
    fn from(value: super::sign_with_legacy_keychain::SignLegacyKeychainContext) -> Self {
        SignKeychainContext {
            network_config: value.network_config,
            global_context: value.global_context,
            signed_transaction_or_signed_delegate_action: value
                .signed_transaction_or_signed_delegate_action,
            on_before_sending_transaction_callback: value.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: value.on_after_sending_transaction_callback,
        }
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
            let access_key_list = block_on(
                    network_config.client().rpc().view_access_key_list(
                        &previous_context.prepopulated_transaction.signer_id,
                        near_kit::Finality::Final.into(),
                    ),
                )
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

        let (nonce, block_hash, block_height) = if previous_context.global_context.offline {
            (
                scope
                    .nonce
                    .wrap_err("Nonce is required to sign a transaction in offline mode")?,
                scope
                    .block_hash
                    .wrap_err("Block Hash is required to sign a transaction in offline mode")?
                    .0,
                scope
                    .block_height
                    .wrap_err("Block Height is required to sign a transaction in offline mode")?,
            )
        } else {
            let access_key_view = block_on(
                    network_config.client().rpc().view_access_key(
                        &previous_context.prepopulated_transaction.signer_id,
                        &account_json.public_key,
                        near_kit::BlockReference::optimistic(),
                    ),
                )
                .into_eyre()
                .wrap_err_with(||
                    format!("Cannot sign a transaction due to an error while fetching the most recent nonce value on network <{}>", network_config.network_name)
                )?;

            (
                access_key_view.nonce + 1,
                access_key_view.block_hash,
                access_key_view.block_height,
            )
        };

        let mut unsigned_transaction = near_kit::Transaction {
            public_key: nk_public_key.clone(),
            block_hash,
            nonce,
            signer_id: previous_context.prepopulated_transaction.signer_id,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            actions: previous_context.prepopulated_transaction.actions,
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        if previous_context.sign_as_delegate_action {
            let max_block_height = block_height
                + scope
                    .meta_transaction_valid_for
                    .unwrap_or(super::META_TRANSACTION_VALID_FOR_DEFAULT);

            let signed_delegate_action = super::get_signed_delegate_action(
                unsigned_transaction,
                &nk_public_key,
                nk_secret_key,
                max_block_height,
            );

            return Ok(Self {
                network_config: previous_context.network_config,
                global_context: previous_context.global_context,
                signed_transaction_or_signed_delegate_action: signed_delegate_action.into(),
                on_before_sending_transaction_callback: previous_context
                    .on_before_sending_transaction_callback,
                on_after_sending_transaction_callback: previous_context
                    .on_after_sending_transaction_callback,
            });
        }

        let mut signed_transaction = unsigned_transaction.sign(&nk_secret_key);

        tracing::info!(
            parent: &tracing::Span::none(),
            "Your transaction was signed successfully.{}",
            crate::common::indent_payload(&format!(
                "\nPublic key: {}\nSignature:  {}\n ",
                nk_public_key,
                signed_transaction.signature
            ))
        );

        (previous_context.on_after_signing_callback)(
            &mut signed_transaction,
            &previous_context.network_config,
        )?;

        Ok(Self {
            network_config: previous_context.network_config,
            global_context: previous_context.global_context,
            signed_transaction_or_signed_delegate_action: signed_transaction.into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
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
        Self {
            network_config: item.network_config,
            global_context: item.global_context,
            signed_transaction_or_signed_delegate_action: item
                .signed_transaction_or_signed_delegate_action,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
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
                CustomType::<u64>::new(
                    "Enter recent block height:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }
}
