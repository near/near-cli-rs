use std::str::FromStr;

use near_primitives::borsh::BorshDeserialize;

use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::network_for_transaction::NetworkForTransactionArgsContext)]
#[interactive_clap(output_context = SignSeedPhraseContext)]
pub struct SignSeedPhrase {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignSeedPhraseContext(super::SubmitContext);

impl SignSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: super::super::network_for_transaction::NetworkForTransactionArgsContext,
        scope: &<SignSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();

        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            scope.seed_phrase_hd_path.clone(),
            scope.master_seed_phrase.clone(),
        )?;

        let signer_secret_key: near_crypto::SecretKey =
            near_crypto::SecretKey::from_str(&key_pair_properties.secret_keypair_str)?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        let rpc_query_response = network_config
            .json_rpc_client()
            .blocking_call_view_access_key(
                &previous_context.relayer_account_id,
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

        let serialize_from_base64 =
            near_primitives::serialize::from_base64(&previous_context.transaction_hash).unwrap();

        let signed_delegate_action =
            near_primitives::delegate_action::SignedDelegateAction::try_from_slice(
                &serialize_from_base64,
            )?;

        let actions = vec![near_primitives::transaction::Action::Delegate(
            signed_delegate_action.clone(),
        )];

        let unsigned_transaction = near_primitives::transaction::Transaction {
            public_key,
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            signer_id: previous_context.relayer_account_id,
            receiver_id: signed_delegate_action.delegate_action.sender_id,
            actions,
        };

        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction (delegate) was signed successfully.");

        Ok(Self(super::SubmitContext {
            network_config: previous_context.network_config,
            signed_transaction,
        }))
    }
}

impl From<SignSeedPhraseContext> for super::SubmitContext {
    fn from(item: SignSeedPhraseContext) -> Self {
        item.0
    }
}

impl SignSeedPhrase {
    fn input_seed_phrase_hd_path(
        _context: &super::super::network_for_transaction::NetworkForTransactionArgsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        Ok(Some(
            inquire::CustomType::new("Enter seed phrase HD Path [if not sure, keep the default]")
                .with_default(crate::types::slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap())
                .prompt()?,
        ))
    }
}
