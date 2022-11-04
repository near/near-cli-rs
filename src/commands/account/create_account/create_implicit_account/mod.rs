use dialoguer::Input;
use std::io::Write;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImplicitAccount {
    #[interactive_clap(subcommand)]
    mode: Mode,
}

impl ImplicitAccount {
    pub async fn process(&self) -> crate::CliResult {
        self.mode.process().await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Choose a mode to create an implicit account
pub enum Mode {
    #[strum_discriminants(strum(
        message = "use-auto-generation  - Use auto-generation to create an implicit account"
    ))]
    ///Use auto-generation to create an implicit account
    UseAutoGeneration(self::SaveImplicitAccount),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "use-ledger           - Use ledger to create an implicit account"
    ))]
    ///Use ledger to create an implicit account
    UseLedger(self::SaveImplicitAccount),
}

impl Mode {
    pub async fn process(&self) -> crate::CliResult {
        let mut file_path = std::path::PathBuf::new();
        let mut file_name = std::path::PathBuf::new();
        let mut buf = String::new();
        match self {
            Mode::UseAutoGeneration(save_implicit_account) => {
                let key_pair_properties = crate::common::generate_keypair().await?;
                buf.push_str(
                    &serde_json::json!({
                        "master_seed_phrase": key_pair_properties.master_seed_phrase,
                        "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path,
                        "implicit_account_id": key_pair_properties.implicit_account_id,
                        "public_key": key_pair_properties.public_key_str,
                        "private_key": key_pair_properties.secret_keypair_str,
                    })
                    .to_string(),
                );
                file_name.push(format!("{}.json", key_pair_properties.implicit_account_id));
                file_path.push(save_implicit_account.save_to_folder.get_folder_path());
            }
            #[cfg(feature = "ledger")]
            Mode::UseLedger(save_implicit_account) => {
                let seed_phrase_hd_path = crate::transaction_signature_options::sign_with_ledger::SignLedger::input_seed_phrase_hd_path();
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
                let public_key_str = format!("ed25519:{}", bs58::encode(&public_key).into_string());
                let implicit_account_id =
                    near_primitives::types::AccountId::try_from(hex::encode(public_key))?;
                buf.push_str(
                    &serde_json::json!({
                        "seed_phrase_hd_path": seed_phrase_hd_path.to_string(),
                        "implicit_account_id": implicit_account_id.to_string(),
                        "public_key": public_key_str,
                    })
                    .to_string(),
                );
                file_name = format!("{}.json", implicit_account_id).into();
                file_path.push(save_implicit_account.save_to_folder.get_folder_path());
            }
        }
        std::fs::create_dir_all(&file_path)?;
        file_path.push(file_name);
        std::fs::File::create(&file_path)
            .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
            .write(buf.as_bytes())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
            })?;
        println!("\nThe file {:?} was saved successfully", &file_path);
        Ok(())
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SaveImplicitAccount {
    #[interactive_clap(named_arg)]
    ///Specify a folder to save the implicit account file
    save_to_folder: SaveToFolder,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SaveToFolder {
    #[interactive_clap(skip_default_input_arg)]
    ///Where to save the implicit account file?
    folder_path: crate::types::path_buf::PathBuf,
}

impl SaveToFolder {
    fn get_folder_path(&self) -> std::path::PathBuf {
        self.folder_path.clone().into()
    }

    fn input_folder_path(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::path_buf::PathBuf> {
        println!();
        let input_folder_path: String = Input::new()
            .with_prompt("Where to save the implicit account file?")
            .with_initial_text(format!(
                "{}/implicit",
                context.0.credentials_home_dir.to_string_lossy()
            ))
            .interact_text()?;
        let folder_path = shellexpand::tilde(&input_folder_path).as_ref().parse()?;
        Ok(folder_path)
    }
}
