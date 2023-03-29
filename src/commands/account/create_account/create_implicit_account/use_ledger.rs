use std::io::Write;

use color_eyre::eyre::Context;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SaveWithLedgerContext)]
pub struct SaveWithLedger {
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Clone)]
pub struct SaveWithLedgerContext(super::SaveImplicitAccountContext);

impl SaveWithLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<SaveWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let previous_context = previous_context.clone();
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                move |folder_path| {
                    let seed_phrase_hd_path = crate::transaction_signature_options::sign_with_ledger::SignLedger::input_seed_phrase_hd_path()?.unwrap();
                    println!(
                        "Please allow getting the PublicKey on Ledger device (HD Path: {})",
                        seed_phrase_hd_path
                    );
                    let public_key = near_ledger::get_public_key(seed_phrase_hd_path.clone().into())
                    .map_err(|near_ledger_error| {
                        color_eyre::Report::msg(format!(
                            "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                            near_ledger_error
                        ))
                    })?;
                    let public_key_str =
                        format!("ed25519:{}", bs58::encode(&public_key).into_string());
                    let implicit_account_id =
                        near_primitives::types::AccountId::try_from(hex::encode(public_key))?;
                    let buf = serde_json::json!({
                        "seed_phrase_hd_path": seed_phrase_hd_path.to_string(),
                        "implicit_account_id": implicit_account_id.to_string(),
                        "public_key": public_key_str,
                    })
                    .to_string();
                    let file_name: std::path::PathBuf =
                        format!("{}.json", implicit_account_id).into();
                    let mut file_path = std::path::PathBuf::new();
                    file_path.push(folder_path);

                    std::fs::create_dir_all(&file_path)?;
                    file_path.push(file_name);
                    std::fs::File::create(&file_path)
                        .wrap_err_with(|| format!("Failed to create file: {:?}", file_path))?
                        .write(buf.as_bytes())
                        .wrap_err_with(|| format!("Failed to write to file: {:?}", file_path))?;
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

impl From<SaveWithLedgerContext> for super::SaveImplicitAccountContext {
    fn from(item: SaveWithLedgerContext) -> Self {
        item.0
    }
}
