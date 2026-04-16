use std::str::FromStr;

use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignSeedPhraseContext)]
pub struct SignSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
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
pub struct SignSeedPhraseContext(super::SubmitContext);

impl SignSeedPhraseContext {
    #[tracing::instrument(name = "Signing the transaction using the seed phrase ...", skip_all)]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Signing the transaction using the seed phrase ...");

        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            scope.seed_phrase_hd_path.clone(),
            scope.master_seed_phrase.clone(),
        )?;

        let signer_secret_key: near_kit::SecretKey =
            near_kit::SecretKey::from_str(&key_pair_properties.secret_keypair_str)?;
        let signer_public_key =
            near_kit::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        let (nonce, block_hash, block_height) = super::resolve_nonce_and_block(
            &previous_context.network_config,
            &previous_context.prepopulated_transaction.signer_id,
            &signer_public_key,
            previous_context.global_context.offline,
            scope.nonce,
            scope.block_hash,
            scope.block_height,
        )?;

        Ok(Self(super::sign_transaction_with_secret_key(
            signer_public_key,
            signer_secret_key,
            previous_context,
            nonce,
            block_hash,
            block_height,
            scope.meta_transaction_valid_for,
        )?))
    }
}

impl From<SignSeedPhraseContext> for super::SubmitContext {
    fn from(item: SignSeedPhraseContext) -> Self {
        item.0
    }
}

impl SignSeedPhrase {
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

    fn input_seed_phrase_hd_path(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        input_seed_phrase_hd_path()
    }
}

pub fn input_seed_phrase_hd_path()
-> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
    Ok(Some(
        CustomType::new("Enter seed phrase HD Path (if you're not sure, keep the default):")
            .with_starting_input("m/44'/397'/0'")
            .prompt()?,
    ))
}
