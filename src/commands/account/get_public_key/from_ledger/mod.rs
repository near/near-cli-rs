use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromLedgerContext)]
pub struct PublicKeyFromLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    connection: LedgerConnectionType,
}

#[derive(Debug, Clone)]
pub struct PublicKeyFromLedgerContext {
    pub global_context: crate::GlobalContext,
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

impl PublicKeyFromLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = PublicKeyFromLedgerContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select Ledger connection type:
pub enum LedgerConnectionType {
    #[strum_discriminants(strum(message = "usb        - Connect to Ledger via USB"))]
    /// Connect to Ledger via USB
    Usb(UsbGetPublicKey),
    #[cfg(feature = "ledger-ble")]
    #[strum_discriminants(strum(message = "bluetooth  - Connect to Ledger via Bluetooth"))]
    /// Connect to Ledger via Bluetooth
    Bluetooth(BluetoothGetPublicKey),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PublicKeyFromLedgerContext)]
#[interactive_clap(output_context = UsbGetPublicKeyContext)]
pub struct UsbGetPublicKey {}

#[derive(Debug, Clone)]
pub struct UsbGetPublicKeyContext;

impl UsbGetPublicKeyContext {
    pub fn from_previous_context(
        previous_context: PublicKeyFromLedgerContext,
        _scope: &<UsbGetPublicKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path;
        eprintln!("Opening the NEAR application... Please approve opening the application");
        near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
        })?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        eprintln!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})"
        );
        let verifying_key = near_ledger::get_public_key(seed_phrase_hd_path.into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                ))
            },
        )?;
        let public_key = near_kit::PublicKey::ed25519_from_bytes(
            verifying_key.to_bytes(),
        );

        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
            previous_context.global_context.verbosity
        {
            eprint!("Public key (printed to stdout): ");
        }
        println!("{public_key}");

        Ok(Self)
    }
}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PublicKeyFromLedgerContext)]
#[interactive_clap(output_context = BleGetPublicKeyContext)]
pub struct BluetoothGetPublicKey {}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone)]
pub struct BleGetPublicKeyContext;

#[cfg(feature = "ledger-ble")]
impl BleGetPublicKeyContext {
    pub fn from_previous_context(
        previous_context: PublicKeyFromLedgerContext,
        _scope: &<BluetoothGetPublicKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path;

        let verifying_key = crate::transaction_signature_options::sign_with_ledger::ble_helpers::ble_connect_and_get_public_key(seed_phrase_hd_path.into())?;
        let public_key = near_kit::PublicKey::ed25519_from_bytes(
            verifying_key.to_bytes(),
        );

        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
            previous_context.global_context.verbosity
        {
            eprint!("Public key (printed to stdout): ");
        }
        println!("{public_key}");

        Ok(Self)
    }
}

impl PublicKeyFromLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
