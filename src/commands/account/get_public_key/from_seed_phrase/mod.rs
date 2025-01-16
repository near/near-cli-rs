use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromSeedPhraseContext)]
pub struct PublicKeyFromSeedPhrase {
    /// Enter the seed-phrase:
    master_seed_phrase: String,
}

#[derive(Debug, Clone)]
pub struct PublicKeyFromSeedPhraseContext;

impl PublicKeyFromSeedPhraseContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path_default = slipped10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            seed_phrase_hd_path_default,
            &scope.master_seed_phrase,
        )?;
        eprintln!("\nPublic key: {}", public_key);

        Ok(Self)
    }
}
