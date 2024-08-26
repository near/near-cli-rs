use std::io::Write;

use color_eyre::eyre::Context;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SaveWithLedgerContext)]
pub struct SaveWithLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Clone)]
pub struct SaveWithLedgerContext(super::SaveImplicitAccountContext);

impl SaveWithLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SaveWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                let seed_phrase_hd_path = scope.seed_phrase_hd_path.clone();
                move |folder_path| {
                    eprintln!(
                        "Opening the NEAR application... Please approve opening the application"
                    );
                    near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
        })?;
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    eprintln!(
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

impl From<SaveWithLedgerContext> for super::SaveImplicitAccountContext {
    fn from(item: SaveWithLedgerContext) -> Self {
        item.0
    }
}

impl SaveWithLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
