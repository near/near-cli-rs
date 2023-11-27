use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select, Text};

use near_ledger::OnlyBlindSigning;
use near_primitives::borsh::BorshSerialize;
use slip10::BIP32Path;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignLedgerContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[allow(dead_code)]
    #[interactive_clap(skip)]
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignLedgerContext {
    network_config: crate::config::NetworkConfig,
    global_context: crate::GlobalContext,
    signed_transaction_or_signed_delegate_action: super::SignedTransactionOrSignedDelegateAction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignLedgerContext {
    fn input_blind_agree() -> color_eyre::eyre::Result<bool> {
        let options: Vec<&str> = vec!["Yes", "No"];

        Ok(
            Select::new("Do you agree to continue with blind signature? ", options)
                .prompt()
                .map(|selected| selected == "Yes")?,
        )
    }

    fn blind_sign_subflow(
        hash: OnlyBlindSigning,
        hd_path: BIP32Path,
        unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> color_eyre::eyre::Result<near_crypto::Signature> {
        eprintln!("\n\nBuffer overflow on Ledger device occured. Transaction is too large for normal signature.");
        eprintln!("\nThe following is Base58-encoded SHA-256 hash of unsigned transaction:");
        eprintln!("{}", hash.0);

        eprintln!(
            "\nUnsigned transaction (serialized as base64):\n{}\n",
            crate::types::transaction::TransactionAsBase64::from(unsigned_transaction)
        );
        eprintln!("Before proceeding with blind signature,");
        eprintln!("you have ability to verify unsigned transaction's details and exact SHA256 correspondence,");
        eprintln!("if you have concerns with trust to current computer, where near CLI command is being run,");
        eprintln!("on another computer(s) with the following helper command on near CLI:");
        eprintln!(
            "$ {} transaction print-transaction unsigned\n\n",
            crate::common::get_near_exec_path()
        );

        eprintln!("Make sure to enable blind sign in Near app's settings on Ledger device\n");
        let agree = Self::input_blind_agree()?;
        if agree {
            eprintln!(
                "Confirm transaction blind signing on your Ledger device (HD Path: {})",
                hd_path,
            );
            let result = near_ledger::blind_sign_transaction(hash, hd_path);
            let signature = result.map_err(|err| {
                match err {
                    near_ledger::NEARLedgerError::BlindSignatureDisabled => {
                        color_eyre::Report::msg("Blind signature is disabled in NEAR app's settings on Ledger device".to_string())
                    },
                    near_ledger::NEARLedgerError::BlindSignatureNotSupported => {
                        color_eyre::Report::msg("Blind signature is not supported by the version of NEAR app installed on Ledger device".to_string())
                    },
                    err => {
                        color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            err
                        ))
                    }
                }
            })?;
            let signature =
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization");

            Ok(signature)
        } else {
            Err(color_eyre::Report::msg("signing with ledger aborted"))
        }
    }

    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();
        let seed_phrase_hd_path: slip10::BIP32Path = scope.seed_phrase_hd_path.clone().into();
        let public_key: near_crypto::PublicKey = scope.signer_public_key.clone().into();

        let (nonce, block_hash) = if previous_context.global_context.offline {
            (
                scope
                    .nonce
                    .wrap_err("Nonce is required to sign a transaction in offline mode")?,
                scope
                    .block_hash
                    .wrap_err("Block Hash is required to sign a transaction in offline mode")?
                    .0,
            )
        } else {
            let rpc_query_response = network_config
                .json_rpc_client()
                .blocking_call_view_access_key(
                    &previous_context.prepopulated_transaction.signer_id,
                    &public_key,
                    near_primitives::types::BlockReference::latest()
                )
                .wrap_err(
                    "Cannot sign a transaction due to an error while fetching the most recent nonce value",
                )?;
            let current_nonce = rpc_query_response
                .access_key_view()
                .wrap_err("Error current_nonce")?
                .nonce;

            (current_nonce + 1, rpc_query_response.block_hash)
        };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: scope.signer_public_key.clone().into(),
            block_hash,
            nonce,
            signer_id: previous_context.prepopulated_transaction.signer_id,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            actions: previous_context.prepopulated_transaction.actions,
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        eprintln!(
            "Confirm transaction signing on your Ledger device (HD Path: {})",
            seed_phrase_hd_path,
        );

        let signature = match near_ledger::sign_transaction(
            unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
            seed_phrase_hd_path.clone(),
        ) {
            Ok(signature) => {
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization")
            }
            Err(near_ledger::NEARLedgerError::BufferOverflow(hash)) => {
                Self::blind_sign_subflow(hash, seed_phrase_hd_path, unsigned_transaction.clone())?
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

impl From<SignLedgerContext> for super::SubmitContext {
    fn from(item: SignLedgerContext) -> Self {
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

impl interactive_clap::FromCli for SignLedger {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignLedger as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.seed_phrase_hd_path.is_none() {
            clap_variant.seed_phrase_hd_path = match Self::input_seed_phrase_hd_path() {
                Ok(Some(seed_phrase_hd_path)) => Some(seed_phrase_hd_path),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let seed_phrase_hd_path = clap_variant
            .seed_phrase_hd_path
            .clone()
            .expect("Unexpected error");

        eprintln!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = match near_ledger::get_public_key(seed_phrase_hd_path.clone().into())
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            }) {
            Ok(public_key) => public_key,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };
        let signer_public_key: crate::types::public_key::PublicKey =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                public_key.to_bytes(),
            ))
            .into();

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
        let block_hash = clap_variant.block_hash;

        let new_context_scope = InteractiveClapContextScopeForSignLedger {
            signer_public_key,
            seed_phrase_hd_path,
            nonce,
            block_hash,
        };
        let output_context =
            match SignLedgerContext::from_previous_context(context, &new_context_scope) {
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

impl SignLedger {
    pub fn input_seed_phrase_hd_path(
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        Ok(Some(
            crate::types::slip10::BIP32Path::from_str(
                &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default):")
                    .with_initial_value("44'/397'/0'/0'/1'")
                    .prompt()
                    .unwrap(),
            )
            .unwrap(),
        ))
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
}
