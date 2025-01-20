#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromSeedPhraseContext)]
pub struct PublicKeyFromSeedPhrase {
    /// Enter the seed-phrase:
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

#[derive(Debug, Clone)]
pub struct PublicKeyFromSeedPhraseContext;

impl PublicKeyFromSeedPhraseContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key = crate::common::get_public_key_from_seed_phrase(
            scope.seed_phrase_hd_path.clone().into(),
            &scope.master_seed_phrase,
        )?;
        eprintln!("\nPublic key: {}", public_key);

        Ok(Self)
    }
}

impl PublicKeyFromSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_seed_phrase::input_seed_phrase_hd_path()
    }
}
