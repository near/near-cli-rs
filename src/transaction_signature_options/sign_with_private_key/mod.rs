use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::CustomType;
use near_primitives::transaction::TransactionV0;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignPrivateKeyContext)]
pub struct SignPrivateKey {
    #[interactive_clap(long)]
    /// Enter sender (signer) public key:
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
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
        let network_config = previous_context.network_config.clone();
        let signer_secret_key: near_crypto::SecretKey = scope.signer_private_key.clone().into();
        let public_key: near_crypto::PublicKey = scope.signer_public_key.clone().into();

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
            let rpc_query_response = network_config
                .json_rpc_client()
                .blocking_call_view_access_key(
                    &previous_context.prepopulated_transaction.signer_id,
                    &public_key,
                    near_primitives::types::BlockReference::latest()
                )
                .wrap_err_with(||
                    format!("Cannot sign a transaction due to an error while fetching the most recent nonce value on network <{}>", network_config.network_name)
                )?;
            (
                rpc_query_response
                    .access_key_view()
                    .wrap_err("Error current_nonce")?
                    .nonce
                    + 1,
                rpc_query_response.block_hash,
                rpc_query_response.block_height,
            )
        };

        let mut unsigned_transaction =
            near_primitives::transaction::Transaction::V0(TransactionV0 {
                public_key: public_key.clone(),
                block_hash,
                nonce,
                signer_id: previous_context.prepopulated_transaction.signer_id,
                receiver_id: previous_context.prepopulated_transaction.receiver_id,
                actions: previous_context.prepopulated_transaction.actions,
            });

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());

        if network_config.meta_transaction_relayer_url.is_some() {
            let max_block_height = block_height
                + scope
                    .meta_transaction_valid_for
                    .unwrap_or(super::META_TRANSACTION_VALID_FOR_DEFAULT);

            let signed_delegate_action = super::get_signed_delegate_action(
                unsigned_transaction,
                &public_key,
                signer_secret_key,
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

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction was signed successfully.");
        eprintln!("Public key: {}", scope.signer_public_key);
        eprintln!("Signature: {}", signature);

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
