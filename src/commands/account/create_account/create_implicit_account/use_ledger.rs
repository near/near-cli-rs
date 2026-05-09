use std::io::Write;

use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SaveWithLedgerContext)]
pub struct SaveWithLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    connection: LedgerConnectionType,
}

#[derive(Clone)]
pub struct SaveWithLedgerContext {
    pub global_context: crate::GlobalContext,
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

impl SaveWithLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SaveWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SaveWithLedgerContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select Ledger connection type:
pub enum LedgerConnectionType {
    #[strum_discriminants(strum(message = "usb        - Connect to Ledger via USB"))]
    /// Connect to Ledger via USB
    Usb(UsbSaveWithLedger),
    #[cfg(feature = "ledger-ble")]
    #[strum_discriminants(strum(message = "bluetooth  - Connect to Ledger via Bluetooth"))]
    /// Connect to Ledger via Bluetooth
    Bluetooth(BluetoothSaveWithLedger),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SaveWithLedgerContext)]
#[interactive_clap(output_context = UsbSaveWithLedgerContext)]
pub struct UsbSaveWithLedger {
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[derive(Clone)]
pub struct UsbSaveWithLedgerContext(super::SaveImplicitAccountContext);

impl UsbSaveWithLedgerContext {
    pub fn from_previous_context(
        previous_context: SaveWithLedgerContext,
        _scope: &<UsbSaveWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                let seed_phrase_hd_path = previous_context.seed_phrase_hd_path.clone();
                let verbosity = previous_context.global_context.verbosity;
                move |folder_path| {
                    eprintln!(
                        "Opening the NEAR application... Please approve opening the application"
                    );
                    near_ledger::open_near_application().map_err(|ledger_error| {
                        color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
                    })?;
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    eprintln!(
                        "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})"
                    );
                    let public_key = near_ledger::get_public_key(seed_phrase_hd_path.clone().into())
                    .map_err(|near_ledger_error| {
                        color_eyre::Report::msg(format!(
                            "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                        ))
                    })?;
                    save_implicit_account(&seed_phrase_hd_path, public_key, folder_path, &verbosity)
                }
            });
        Ok(Self(super::SaveImplicitAccountContext {
            config: previous_context.global_context.config,
            on_after_getting_folder_path_callback,
        }))
    }
}

impl From<UsbSaveWithLedgerContext> for super::SaveImplicitAccountContext {
    fn from(item: UsbSaveWithLedgerContext) -> Self {
        item.0
    }
}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SaveWithLedgerContext)]
#[interactive_clap(output_context = BleSaveWithLedgerContext)]
pub struct BluetoothSaveWithLedger {
    #[interactive_clap(named_arg)]
    /// Specify a folder to save the implicit account file
    save_to_folder: super::SaveToFolder,
}

#[cfg(feature = "ledger-ble")]
#[derive(Clone)]
pub struct BleSaveWithLedgerContext(super::SaveImplicitAccountContext);

#[cfg(feature = "ledger-ble")]
impl BleSaveWithLedgerContext {
    pub fn from_previous_context(
        previous_context: SaveWithLedgerContext,
        _scope: &<BluetoothSaveWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_folder_path_callback: super::OnAfterGettingFolderPathCallback =
            std::sync::Arc::new({
                let seed_phrase_hd_path = previous_context.seed_phrase_hd_path.clone();
                let verbosity = previous_context.global_context.verbosity;
                move |folder_path| {
                    let public_key = crate::transaction_signature_options::sign_with_ledger::ble_helpers::ble_connect_and_get_public_key(seed_phrase_hd_path.clone().into())?;
                    save_implicit_account(&seed_phrase_hd_path, public_key, folder_path, &verbosity)
                }
            });
        Ok(Self(super::SaveImplicitAccountContext {
            config: previous_context.global_context.config,
            on_after_getting_folder_path_callback,
        }))
    }
}

#[cfg(feature = "ledger-ble")]
impl From<BleSaveWithLedgerContext> for super::SaveImplicitAccountContext {
    fn from(item: BleSaveWithLedgerContext) -> Self {
        item.0
    }
}

fn save_implicit_account(
    seed_phrase_hd_path: &crate::types::slip10::BIP32Path,
    public_key: ed25519_dalek::VerifyingKey,
    folder_path: &std::path::PathBuf,
    verbosity: &crate::Verbosity,
) -> crate::CliResult {
    let public_key_str = format!("ed25519:{}", bs58::encode(&public_key).into_string());
    let implicit_account_id = near_kit::AccountId::try_from(hex::encode(public_key))?;
    let buf = serde_json::json!({
        "seed_phrase_hd_path": seed_phrase_hd_path.to_string(),
        "implicit_account_id": implicit_account_id.to_string(),
        "public_key": public_key_str,
    })
    .to_string();
    let file_name: std::path::PathBuf = format!("{implicit_account_id}.json").into();
    let mut file_path = std::path::PathBuf::new();
    file_path.push(folder_path);

    std::fs::create_dir_all(&file_path)?;
    file_path.push(file_name);
    std::fs::File::create(&file_path)
        .wrap_err_with(|| format!("Failed to create file: {file_path:?}"))?
        .write(buf.as_bytes())
        .wrap_err_with(|| format!("Failed to write to file: {file_path:?}"))?;

    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
        eprintln!("\nThe file {file_path:?} was saved successfully");
    }

    Ok(())
}

impl SaveWithLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
