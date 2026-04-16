use color_eyre::eyre::WrapErr;
use near_crypto::Signature;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::FinalSignNep413Context)]
#[interactive_clap(output_context = SignLedgerContext)]
pub struct SignLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    connection: LedgerConnectionType,
}

#[derive(Debug, Clone)]
pub struct SignLedgerContext {
    pub final_context: super::super::FinalSignNep413Context,
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

impl SignLedgerContext {
    pub fn from_previous_context(
        previous_context: super::super::FinalSignNep413Context,
        scope: &<SignLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            final_context: previous_context,
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SignLedgerContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select Ledger connection type:
pub enum LedgerConnectionType {
    #[strum_discriminants(strum(message = "usb        - Connect to Ledger via USB"))]
    /// Connect to Ledger via USB
    Usb(UsbSignNep413),
    #[cfg(feature = "ledger-ble")]
    #[strum_discriminants(strum(message = "bluetooth  - Connect to Ledger via Bluetooth"))]
    /// Connect to Ledger via Bluetooth
    Bluetooth(BluetoothSignNep413),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignLedgerContext)]
#[interactive_clap(output_context = UsbSignNep413Context)]
pub struct UsbSignNep413 {}

#[derive(Debug, Clone)]
pub struct UsbSignNep413Context;

impl UsbSignNep413Context {
    pub fn from_previous_context(
        previous_context: SignLedgerContext,
        _scope: &<UsbSignNep413 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path;

        eprintln!("Opening the NEAR application... Please approve opening the application");
        near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!(
                "An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"
            ))
        })?;

        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            near_ledger::get_public_key(seed_phrase_hd_path.clone().into())
                .map_err(|err| color_eyre::eyre::eyre!("Ledger get_public_key error: {err:?}"))?
                .to_bytes(),
        ));

        std::thread::sleep(std::time::Duration::from_secs(1));

        eprintln!(
            "Please approve the message signing on your Ledger device (HD Path: {seed_phrase_hd_path})"
        );

        let signature_bytes = near_ledger::sign_message_nep413(
            &previous_context.final_context.payload.into(),
            seed_phrase_hd_path.into(),
        )
        .map_err(|err| color_eyre::eyre::eyre!("Ledger signing error: {:?}", err))?;

        let signature = Signature::from_parts(near_crypto::KeyType::ED25519, &signature_bytes)
            .wrap_err("Signature is not expected to fail on deserialization")?;

        let signed_message = super::super::SignedMessage {
            account_id: previous_context.final_context.signer_id.to_string(),
            public_key: public_key.to_string(),
            signature: signature.to_string(),
        };
        (previous_context.final_context.on_after_signing_callback)(signed_message)?;

        Ok(Self)
    }
}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignLedgerContext)]
#[interactive_clap(output_context = BleSignNep413Context)]
pub struct BluetoothSignNep413 {}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone)]
pub struct BleSignNep413Context;

#[cfg(feature = "ledger-ble")]
impl BleSignNep413Context {
    pub fn from_previous_context(
        previous_context: SignLedgerContext,
        _scope: &<BluetoothSignNep413 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = previous_context.seed_phrase_hd_path;

        let (verifying_key, signature_bytes) =
            crate::transaction_signature_options::sign_with_ledger::ble_helpers::ble_get_public_key_and_sign_nep413(
                seed_phrase_hd_path.clone().into(),
                previous_context.final_context.payload.into(),
            )?;

        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            verifying_key.to_bytes(),
        ));

        let signature = Signature::from_parts(near_crypto::KeyType::ED25519, &signature_bytes)
            .wrap_err("Signature is not expected to fail on deserialization")?;

        let signed_message = super::super::SignedMessage {
            account_id: previous_context.final_context.signer_id.to_string(),
            public_key: public_key.to_string(),
            signature: signature.to_string(),
        };
        (previous_context.final_context.on_after_signing_callback)(signed_message)?;

        Ok(Self)
    }
}

impl SignLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &super::super::FinalSignNep413Context,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
