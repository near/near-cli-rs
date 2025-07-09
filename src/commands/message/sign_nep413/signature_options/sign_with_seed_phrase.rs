use std::str::FromStr;
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SignNep413Context)]
#[interactive_clap(output_context = SignSeedPhraseContext)]
pub struct SignSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

#[derive(Debug, Clone)]
pub struct SignSeedPhraseContext;

impl SignSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: super::super::SignNep413Context,
        scope: &<SignSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            scope.seed_phrase_hd_path.clone(),
            scope.master_seed_phrase.clone(),
        )?;
        let secret_key = near_crypto::SecretKey::from_str(&key_pair_properties.secret_keypair_str)?;

        let signature = super::super::sign_nep413_payload(&previous_context.payload, &secret_key)?;

        let signed_message = super::super::SignedMessage {
            account_id: previous_context.signer_id.to_string(),
            public_key: key_pair_properties.public_key_str,
            signature: signature.to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&signed_message)?);
        Ok(Self)
    }
}

impl SignSeedPhrase {
    fn input_seed_phrase_hd_path(
        _context: &super::super::SignNep413Context,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_seed_phrase::input_seed_phrase_hd_path()
    }
}
