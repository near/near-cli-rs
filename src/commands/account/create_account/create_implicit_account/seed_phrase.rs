use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SeedPhrase {
    ///Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    ///Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

impl SeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::slip10::BIP32Path> {
        Ok(crate::types::slip10::BIP32Path::from_str(
            &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                .with_initial_value("m/44'/397'/0'")
                .prompt()
                .unwrap(),
        )
        .unwrap())
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
