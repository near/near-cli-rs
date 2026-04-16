use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::CustomType;

use crate::common::blocking_view_access_key;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignPrivateKeyContext)]
pub struct SignPrivateKey {
    /// Enter sender (signer) private (secret) key:
    pub signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_height: Option<near_primitives::types::BlockHeight>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    pub submit: super::Submit,
}

#[derive(Clone)]
pub struct SignPrivateKeyContext {
    network_config: crate::config::NetworkConfig,
    global_context: crate::GlobalContext,
    signed_transaction_or_signed_delegate_action: super::SignedTransactionOrSignedDelegateAction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignPrivateKeyContext {
    #[tracing::instrument(
        name = "Signing the transaction with a plaintext private key ...",
        skip_all
    )]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Signing the transaction with a plaintext private key ...");

        let network_config = previous_context.network_config.clone();
        let signer_secret_key: near_crypto::SecretKey = scope.signer_private_key.clone().into();
        let public_key = signer_secret_key.public_key();

        let nk_public_key = crate::common::to_nk_public_key(&public_key);
        let nk_secret_key = crate::common::to_nk_secret_key(&signer_secret_key);

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
            let access_key_view = blocking_view_access_key(
                    &network_config,
                    &previous_context.prepopulated_transaction.signer_id,
                    &public_key,
                    near_primitives::types::BlockReference::latest(),
                )
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

impl From<SignPrivateKeyContext> for super::SubmitContext {
    fn from(item: SignPrivateKeyContext) -> Self {
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

impl SignPrivateKey {
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
    ) -> color_eyre::eyre::Result<Option<near_primitives::types::BlockHeight>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<near_primitives::types::BlockHeight>::new(
                    "Enter recent block height:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }
}
