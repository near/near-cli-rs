use std::io::Write;

use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod save_to_keychain;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SaveWithUseAutoGenerationContext)]
pub struct SaveWithUseAutoGeneration {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Clone)]
pub struct SaveWithUseAutoGenerationContext {
    file_context: super::SaveImplicitAccountContext,
    global_context: crate::GlobalContext,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SaveWithUseAutoGenerationContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Where do you want to save the generated implicit account?
pub enum SaveMode {
    #[strum_discriminants(strum(
        message = "save-to-keychain - Save securely in the system keychain"
    ))]
    /// Save securely in the system keychain without a plaintext export
    SaveToKeychain(save_to_keychain::SaveToKeychain),
    #[strum_discriminants(strum(message = "save-to-folder   - Export a plaintext account file"))]
    /// Specify a folder to save the implicit account file
    SaveToFolder(super::SaveToFolder),
}

impl SaveWithUseAutoGenerationContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<SaveWithUseAutoGeneration as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                move |folder_path| {
                    let key_pair_properties = crate::common::generate_keypair()?;
                    let buf = serde_json::json!({
                        "master_seed_phrase": key_pair_properties.master_seed_phrase,
                        "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path,
                        "implicit_account_id": key_pair_properties.implicit_account_id,
                        "public_key": key_pair_properties.public_key_str,
                        "private_key": key_pair_properties.secret_keypair_str,
                    })
                    .to_string();
                    let mut file_path = std::path::PathBuf::new();
                    let mut file_name = std::path::PathBuf::new();
                    file_name.push(format!("{}.json", key_pair_properties.implicit_account_id));
                    file_path.push(folder_path);

                    std::fs::create_dir_all(&file_path)?;
                    file_path.push(file_name);
                    std::fs::File::create(&file_path)
                        .wrap_err_with(|| format!("Failed to create file: {file_path:?}"))?
                        .write(buf.as_bytes())
                        .wrap_err_with(|| format!("Failed to write to file: {folder_path:?}"))?;

                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        previous_context.verbosity
                    {
                        eprintln!("\nThe file {file_path:?} was saved successfully");
                    }

                    Ok(())
                }
            });
        Ok(Self {
            file_context: super::SaveImplicitAccountContext {
                config: previous_context.config.clone(),
                on_after_getting_folder_path_callback,
            },
            global_context: previous_context,
        })
    }
}

impl From<SaveWithUseAutoGenerationContext> for super::SaveImplicitAccountContext {
    fn from(item: SaveWithUseAutoGenerationContext) -> Self {
        item.file_context
    }
}
