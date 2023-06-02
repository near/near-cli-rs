use std::io::Write;
use std::str::FromStr;

use color_eyre::eyre::Context;
use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SaveWithSeedPhraseContext)]
pub struct SaveWithSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Clone)]
struct SaveWithSeedPhraseContext(super::SaveImplicitAccountContext);

impl SaveWithSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SaveWithSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::get_key_pair_properties_from_seed_phrase(
                scope.seed_phrase_hd_path.clone(),
                scope.master_seed_phrase.clone(),
            )?;
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                move |folder_path| {
                    let mut file_path = std::path::PathBuf::new();
                    let mut file_name = std::path::PathBuf::new();
                    let buf = serde_json::json!({
                        "master_seed_phrase": key_pair_properties.master_seed_phrase,
                        "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path,
                        "implicit_account_id": key_pair_properties.implicit_account_id,
                        "public_key": key_pair_properties.public_key_str,
                        "private_key": key_pair_properties.secret_keypair_str,
                    })
                    .to_string();
                    file_name.push(format!("{}.json", key_pair_properties.implicit_account_id));
                    file_path.push(folder_path);

                    std::fs::create_dir_all(&file_path)?;
                    file_path.push(file_name);
                    std::fs::File::create(&file_path)
                        .wrap_err_with(|| format!("Failed to create file: {:?}", file_path))?
                        .write(buf.as_bytes())
                        .wrap_err_with(|| format!("Failed to write to file: {:?}", file_path))?;
                    eprintln!("\nThe file {:?} was saved successfully", &file_path);

                    Ok(())
                }
            });
        Ok(Self(super::SaveImplicitAccountContext {
            config: previous_context.config,
            on_after_getting_folder_path_callback,
        }))
    }
}

impl From<SaveWithSeedPhraseContext> for super::SaveImplicitAccountContext {
    fn from(item: SaveWithSeedPhraseContext) -> Self {
        item.0
    }
}

impl SaveWithSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        Ok(Some(
            crate::types::slip10::BIP32Path::from_str(
                &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default):")
                    .with_initial_value("m/44'/397'/0'")
                    .prompt()
                    .unwrap(),
            )
            .unwrap(),
        ))
    }
}
