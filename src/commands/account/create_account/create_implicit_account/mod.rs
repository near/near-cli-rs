use inquire::Text;
use std::io::Write;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod seed_phrase;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImplicitAccount {
    #[interactive_clap(subcommand)]
    mode: Mode,
}

// impl ImplicitAccount {
//     pub async fn process(&self) -> crate::CliResult {
//         self.mode.process().await
//     }
// }

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a mode to create an implicit account
pub enum Mode {
    #[strum_discriminants(strum(
        message = "use-auto-generation  - Use auto-generation to create an implicit account"
    ))]
    /// Use auto-generation to create an implicit account
    UseAutoGeneration(self::SaveImplicitAccount),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "use-ledger           - Use ledger to create an implicit account"
    ))]
    /// Use ledger to create an implicit account
    UseLedger(self::SaveImplicitAccount),
    #[strum_discriminants(strum(
        message = "use-seed-phrase      - Use seed phrase to create an implicit account"
    ))]
    /// Use seed phrase to create an implicit account
    UseSeedPhrase(self::seed_phrase::SeedPhrase),
}

#[derive(Debug, Clone)]
pub struct ModeContext {
    config: crate::config::Config,
    mode: ModeDiscriminants,
}

impl ModeContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Mode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            mode: scope.clone(),
        })
    }
}


impl Mode {
    pub async fn process(&self) -> crate::CliResult {
        let mut file_path = std::path::PathBuf::new();
        let mut file_name = std::path::PathBuf::new();
        let mut buf = String::new();
        match self {
            Mode::UseAutoGeneration(save_implicit_account) => {
                let key_pair_properties = crate::common::generate_keypair()?;
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
            Mode::UseSeedPhrase(seed_phrase) => {
                let key_pair_properties = seed_phrase.get_key_pair_properties()?;
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
                file_path.push(seed_phrase.get_folder_path());
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
#[interactive_clap(input_context = ModeContext)]
#[interactive_clap(output_context = SaveImplicitAccountContext)]
pub struct SaveImplicitAccount {
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: SaveToFolder,
}

#[derive(Clone)]
pub struct SaveImplicitAccountContext {
    config: crate::config::Config,
    on_after_getting_folder_path_callback: OnAfterGettingFolderPathCallback,
}

impl SaveImplicitAccountContext {
    pub fn from_previous_context(
        previous_context: ModeContext,
        scope: &<SaveImplicitAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        
        let on_after_getting_folder_path_callback;
        Ok(Self {
            config: previous_context.config,
            on_after_getting_folder_path_callback,
        })
    }
}


#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SaveImplicitAccountContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SaveToFolder {
    #[interactive_clap(skip_default_input_arg)]
    /// Where to save the implicit account file?
    folder_path: crate::types::path_buf::PathBuf,
}

impl interactive_clap::FromCli for SaveToFolder {
    type FromCliContext = SaveImplicitAccountContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        if clap_variant.folder_path.is_none() {
            clap_variant.folder_path = match Self::input_folder_path(&context) {
                Ok(Some(folder_path)) => Some(folder_path),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => {
                    return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                }
            };
        };
        let folder_path = clap_variant.folder_path.clone().expect("Unexpected error");




        interactive_clap::ResultFromCli::Ok(clap_variant)
    }
}

impl SaveToFolder {
    fn get_folder_path(&self) -> std::path::PathBuf {
        self.folder_path.clone().into()
    }

    fn input_folder_path(
        context: &SaveImplicitAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        println!();
        let input_folder_path: String = Text::new("Where to save the implicit account file?")
            .with_initial_value(
                format!(
                    "{}/implicit",
                    context.config.credentials_home_dir.to_string_lossy()
                )
                .as_str(),
            )
            .prompt()?;
        let folder_path = shellexpand::tilde(&input_folder_path).as_ref().parse()?;
        Ok(Some(folder_path))
    }
}

pub type OnAfterGettingFolderPathCallback = std::sync::Arc<
    dyn Fn(
        &std::path::PathBuf,
    ) -> crate::CliResult,
>;
