use std::io::Write;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = super::SaveImplicitAccountContext)]
pub struct SaveWithUseAutoGeneration {
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Clone)]
struct SaveWithUseAutoGenerationContext(super::SaveImplicitAccountContext);

impl SaveWithUseAutoGenerationContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<SaveWithUseAutoGeneration as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let previous_context = previous_context.clone();
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
                        .map_err(|err| {
                            color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                        })?
                        .write(buf.as_bytes())
                        .map_err(|err| {
                            color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                        })?;
                    println!("\nThe file {:?} was saved successfully", &file_path);

                    Ok(())
                }
            });
        Ok(Self(super::SaveImplicitAccountContext {
            config: previous_context.0,
            on_after_getting_folder_path_callback,
        }))
    }
}

impl From<SaveWithUseAutoGenerationContext> for super::SaveImplicitAccountContext {
    fn from(item: SaveWithUseAutoGenerationContext) -> Self {
        item.0
    }
}