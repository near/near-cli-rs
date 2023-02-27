use inquire::Text;
use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = super::SubmitContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(skip)]
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignLedgerContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();
        let seed_phrase_hd_path: slip10::BIP32Path = scope.seed_phrase_hd_path.clone().into();

        let online_signer_access_key_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: previous_context.transaction.signer_id.clone(),
                        public_key: scope.signer_public_key.clone().into(),
                    },
                },
            ))
            .map_err(|err| {
                println!("\nYour transaction was not successfully signed.\n");
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                online_signer_access_key,
            ) = online_signer_access_key_response.kind
            {
                online_signer_access_key.nonce
            } else {
                return Err(color_eyre::Report::msg("Error current_nonce".to_string()));
            };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: scope.signer_public_key.clone().into(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        println!(
            "Confirm transaction signing on your Ledger device (HD Path: {})",
            seed_phrase_hd_path,
        );

        let signature = match near_ledger::sign_transaction(
            unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
            seed_phrase_hd_path,
        ) {
            Ok(signature) => {
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization")
            }
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "Error occurred while signing the transaction: {:?}",
                    near_ledger_error
                )));
            }
        };
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

impl From<SignLedgerContext> for super::SubmitContext {
    fn from(item: SignLedgerContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignLedger {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignLedger as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let seed_phrase_hd_path = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path.clone())
        {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = near_ledger::get_public_key(seed_phrase_hd_path.clone().into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            },
        )?;
        let signer_public_key: crate::types::public_key::PublicKey =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                public_key.to_bytes(),
            ))
            .into();
        let nonce: Option<u64> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.nonce);
        let block_hash: Option<String> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.block_hash.clone());
        let new_context_scope = InteractiveClapContextScopeForSignLedger {
            signer_public_key: signer_public_key.clone(),
            seed_phrase_hd_path: seed_phrase_hd_path.clone(),
            nonce,
            block_hash: block_hash.clone(),
        };
        let ledger_context =
            SignLedgerContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = super::SubmitContext::from(ledger_context);

        let optional_submit = super::Submit::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
            new_context,
        )?;
        let submit = if let Some(submit) = optional_submit {
            submit
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            seed_phrase_hd_path,
            signer_public_key,
            nonce: None,
            block_hash: None,
            submit,
        }))
    }
}

impl SignLedger {
    pub fn input_seed_phrase_hd_path() -> crate::types::slip10::BIP32Path {
        crate::types::slip10::BIP32Path::from_str(
            &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                .with_initial_value("44'/397'/0'/0'/1'")
                .prompt()
                .unwrap(),
        )
        .unwrap()
    }
}
