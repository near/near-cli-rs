use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = super::KeyPairContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct SeedPhrase {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Debug, Clone)]
pub struct SeedPhraseContext {
    config: crate::config::Config,
    master_seed_phrase: String,
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    key_pair_properties: crate::common::KeyPairProperties,
}

impl SeedPhraseContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::get_key_pair_properties_from_seed_phrase(
                scope.seed_phrase_hd_path.clone(),
                scope.master_seed_phrase.clone(),
            )?;
        Ok(Self {
            config: previous_context.0,
            master_seed_phrase: scope.master_seed_phrase.clone(),
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
            key_pair_properties,
        })
    }
}

impl From<SeedPhraseContext> for super::KeyPairContext {
    fn from(item: SeedPhraseContext) -> Self {
        Self {
            config: item.config,
            key_pair_properties: item.key_pair_properties,
        }
    }
}

impl SeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        Ok(Some(
            crate::types::slip10::BIP32Path::from_str(
                &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                    .with_initial_value("m/44'/397'/0'")
                    .prompt()
                    .unwrap(),
            )
            .unwrap(),
        ))
    }

    pub fn get_key_pair_properties(
        &self,
    ) -> color_eyre::eyre::Result<crate::common::KeyPairProperties> {
        crate::common::get_key_pair_properties_from_seed_phrase(
            self.seed_phrase_hd_path.clone(),
            self.master_seed_phrase.clone(),
        )
    }

    pub fn get_folder_path(&self) -> std::path::PathBuf {
        self.save_to_folder.get_folder_path()
    }
}
