use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignPrivateKeyContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignPrivateKey {
    #[interactive_clap(long)]
    /// Enter sender (signer) public key
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    /// Enter sender (signer) private (secret) key
    pub signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    pub submit: super::Submit,
}

#[derive(Clone)]
pub struct SignPrivateKeyContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();
        let signer_secret_key: near_crypto::SecretKey = scope.signer_private_key.clone().into();
        let public_key: near_crypto::PublicKey = scope.signer_public_key.clone().into();

        let rpc_query_response = network_config
            .json_rpc_client()
            .blocking_call_view_access_key(
                &previous_context.transaction.signer_id,
                &public_key,
                near_primitives::types::Finality::Final.into(),
            )
            .map_err(|err| {
                println!("\nYour transaction was not successfully signed.\n");
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce = rpc_query_response
            .access_key_view()
            .wrap_err("Error current_nonce")?
            .nonce;

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: scope.signer_public_key.clone().into(),
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        //XXX print unsigned transaction

        //XXX do you want to sign transaction?

        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        for action in signed_transaction.transaction.actions.iter() {
            if let near_primitives::transaction::Action::FunctionCall(_) = action {
                println!("\nSigned transaction:\n");
                crate::common::print_transaction(signed_transaction.transaction.clone());
            }
        }

        println!("\nYour transaction was signed successfully.");
        println!("Public key: {}", scope.signer_public_key);
        println!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction,
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
            signed_transaction: item.signed_transaction,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignPrivateKey {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignPrivateKey as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.signer_public_key.is_none() {
            clap_variant.signer_public_key = match Self::input_signer_public_key(&context) {
                Ok(Some(signer_public_key)) => Some(signer_public_key),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let signer_public_key = clap_variant
            .signer_public_key
            .clone()
            .expect("Unexpected error");
        if clap_variant.signer_private_key.is_none() {
            clap_variant.signer_private_key = match Self::input_signer_private_key(&context) {
                Ok(Some(signer_private_key)) => Some(signer_private_key),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let signer_private_key = clap_variant
            .signer_private_key
            .clone()
            .expect("Unexpected error");
        if clap_variant.nonce.is_none() {
            clap_variant.nonce = match Self::input_nonce(&context) {
                Ok(optional_nonce) => optional_nonce,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let nonce = clap_variant.nonce;
        if clap_variant.block_hash.is_none() {
            clap_variant.block_hash = match Self::input_block_hash(&context) {
                Ok(optional_block_hash) => optional_block_hash,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let block_hash = clap_variant.block_hash.clone();

        let new_context_scope = InteractiveClapContextScopeForSignPrivateKey {
            signer_public_key,
            signer_private_key,
            nonce,
            block_hash,
        };
        let output_context =
            match SignPrivateKeyContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(submit) => {
                clap_variant.submit = Some(submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_submit) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_submit, err) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl SignPrivateKey {
    pub fn input_nonce(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(None)
    }

    pub fn input_block_hash(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(None)
    }
}
