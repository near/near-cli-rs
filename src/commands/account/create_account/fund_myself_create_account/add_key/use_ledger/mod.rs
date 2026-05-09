use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddAccessWithLedgerContext)]
pub struct AddAccessWithLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    connection: LedgerConnectionType,
}

#[derive(Clone)]
pub struct AddAccessWithLedgerContext {
    pub global_context: crate::GlobalContext,
    pub new_account_id: near_kit::AccountId,
    pub initial_balance: crate::types::near_token::NearToken,
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

impl AddAccessWithLedgerContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<AddAccessWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = AddAccessWithLedgerContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select Ledger connection type:
pub enum LedgerConnectionType {
    #[strum_discriminants(strum(message = "usb        - Connect to Ledger via USB"))]
    /// Connect to Ledger via USB
    Usb(UsbAddAccessWithLedger),
    #[cfg(feature = "ledger-ble")]
    #[strum_discriminants(strum(message = "bluetooth  - Connect to Ledger via Bluetooth"))]
    /// Connect to Ledger via Bluetooth
    Bluetooth(BluetoothAddAccessWithLedger),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AddAccessWithLedgerContext)]
#[interactive_clap(output_context = UsbAddAccessContext)]
pub struct UsbAddAccessWithLedger {
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[derive(Clone)]
pub struct UsbAddAccessContext(super::super::AccountPropertiesContext);

impl UsbAddAccessContext {
    pub fn from_previous_context(
        previous_context: AddAccessWithLedgerContext,
        _scope: &<UsbAddAccessWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path.clone();
        eprintln!("Opening the NEAR application... Please approve opening the application");
        near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
        })?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        eprintln!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})"
        );
        let public_key = near_ledger::get_public_key(seed_phrase_hd_path.into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                ))
            },
        )?;
        let public_key = near_kit::PublicKey::ed25519_from_bytes(public_key.to_bytes());

        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self(super::super::AccountPropertiesContext {
            global_context: previous_context.global_context,
            account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
        }))
    }
}

impl From<UsbAddAccessContext> for super::super::AccountPropertiesContext {
    fn from(item: UsbAddAccessContext) -> Self {
        item.0
    }
}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AddAccessWithLedgerContext)]
#[interactive_clap(output_context = BleAddAccessContext)]
pub struct BluetoothAddAccessWithLedger {
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[cfg(feature = "ledger-ble")]
#[derive(Clone)]
pub struct BleAddAccessContext(super::super::AccountPropertiesContext);

#[cfg(feature = "ledger-ble")]
impl BleAddAccessContext {
    pub fn from_previous_context(
        previous_context: AddAccessWithLedgerContext,
        _scope: &<BluetoothAddAccessWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path.clone();

        let public_key = crate::transaction_signature_options::sign_with_ledger::ble_helpers::ble_connect_and_get_public_key(seed_phrase_hd_path.into())?;
        let public_key = near_kit::PublicKey::ed25519_from_bytes(public_key.to_bytes());

        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self(super::super::AccountPropertiesContext {
            global_context: previous_context.global_context,
            account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
        }))
    }
}

#[cfg(feature = "ledger-ble")]
impl From<BleAddAccessContext> for super::super::AccountPropertiesContext {
    fn from(item: BleAddAccessContext) -> Self {
        item.0
    }
}

impl AddAccessWithLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &super::super::NewAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
