use color_eyre::eyre::WrapErr;
use inquire::CustomType;
use near_ledger::NEARLedgerError;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[cfg(feature = "ledger-ble")]
pub mod ble_helpers;

const SW_BUFFER_OVERFLOW: &str = "0x6990";
const ERR_OVERFLOW_MEMO: &str = "Buffer overflow on Ledger device occurred. \
Transaction is too large for signature. \
This is resolved in https://github.com/dj8yfo/app-near-rs . \
The status is tracked in `About` section.";

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignLedgerContext)]
pub struct SignLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    block_height: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    connection: LedgerConnectionType,
}

#[derive(Clone)]
pub struct SignLedgerContext {
    pub transaction_context: crate::commands::TransactionContext,
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    pub nonce: Option<u64>,
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    pub block_height: Option<u64>,
    pub meta_transaction_valid_for: Option<u64>,
}

impl SignLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            transaction_context: previous_context,
            seed_phrase_hd_path: scope.seed_phrase_hd_path.clone(),
            nonce: scope.nonce,
            block_hash: scope.block_hash,
            block_height: scope.block_height,
            meta_transaction_valid_for: scope.meta_transaction_valid_for,
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
    Usb(UsbConnection),
    #[cfg(feature = "ledger-ble")]
    #[strum_discriminants(strum(message = "bluetooth  - Connect to Ledger via Bluetooth"))]
    /// Connect to Ledger via Bluetooth
    Bluetooth(BluetoothConnection),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignLedgerContext)]
#[interactive_clap(output_context = UsbConnectionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct UsbConnection {
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct UsbConnectionContext(super::SubmitContext);

impl From<UsbConnectionContext> for super::SubmitContext {
    fn from(item: UsbConnectionContext) -> Self {
        item.0
    }
}

impl interactive_clap::FromCli for UsbConnection {
    type FromCliContext = SignLedgerContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<UsbConnection as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        let seed_phrase_hd_path = context.seed_phrase_hd_path.clone();
        let verbosity = context.transaction_context.global_context.verbosity;

        if let crate::Verbosity::Quiet = verbosity {
            println!("Opening the NEAR application... Please approve opening the application");
        }
        tracing::info!(
            parent: &tracing::Span::none(),
            "Opening the NEAR application... Please approve opening the application"
        );

        if let Err(err) = near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
        }) {
            return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        if let crate::Verbosity::Quiet = verbosity {
            println!(
                "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})"
            );
        }
        tracing::info!(
            parent: &tracing::Span::none(),
            "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})\n{}",
            crate::common::indent_payload(" ")
        );

        let public_key =
            match near_ledger::get_public_key(seed_phrase_hd_path.clone().into()).map_err(
                |near_ledger_error| {
                    color_eyre::Report::msg(format!(
                        "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                    ))
                },
            ) {
                Ok(public_key) => public_key,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        let signer_public_key: crate::types::public_key::PublicKey =
            near_kit::PublicKey::ed25519_from_bytes(public_key.to_bytes()).into();

        let output_context = match sign_transaction_with_usb(
            &context.transaction_context,
            &signer_public_key,
            &seed_phrase_hd_path,
            context.nonce,
            context.block_hash,
            context.block_height,
            context.meta_transaction_valid_for,
        ) {
            Ok(ctx) => ctx,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(submit) => {
                clap_variant.submit = Some(submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_submit) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_submit, err) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

/// Shared Ledger signing logic, parameterized by transport closures.
///
/// `sign_tx_fn` signs a serialized transaction, `sign_delegate_fn` signs a
/// serialized delegate action. USB passes the `near_ledger::` free functions,
/// BLE passes methods on a `BleSession`.
#[allow(clippy::too_many_arguments)]
fn sign_transaction_with_ledger(
    previous_context: &crate::commands::TransactionContext,
    signer_public_key: &crate::types::public_key::PublicKey,
    seed_phrase_hd_path: &crate::types::slip10::BIP32Path,
    nonce: Option<u64>,
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    block_height: Option<u64>,
    meta_transaction_valid_for: Option<u64>,
    sign_tx_fn: impl Fn(&[u8], near_slip10::BIP32Path) -> Result<Vec<u8>, NEARLedgerError>,
    sign_delegate_fn: impl Fn(&[u8], near_slip10::BIP32Path) -> Result<Vec<u8>, NEARLedgerError>,
) -> color_eyre::eyre::Result<super::SubmitContext> {
    let network_config = previous_context.network_config.clone();
    let seed_phrase_hd_path_raw: near_slip10::BIP32Path = seed_phrase_hd_path.clone().into();
    let public_key: near_kit::PublicKey = signer_public_key.clone().into();

    let nk_public_key = public_key.clone();

    let (nonce, block_hash, block_height) = super::resolve_nonce_and_block(
        &network_config,
        &previous_context.prepopulated_transaction.signer_id,
        &public_key,
        previous_context.global_context.offline,
        nonce,
        block_hash,
        block_height,
    )?;

    let mut unsigned_transaction = near_kit::Transaction {
        public_key: nk_public_key.clone(),
        block_hash,
        nonce,
        signer_id: previous_context.prepopulated_transaction.signer_id.clone(),
        receiver_id: previous_context
            .prepopulated_transaction
            .receiver_id
            .clone(),
        actions: previous_context.prepopulated_transaction.actions.clone(),
    };

    (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

    if previous_context.sign_as_delegate_action {
        let max_block_height = block_height
            + meta_transaction_valid_for.unwrap_or(super::META_TRANSACTION_VALID_FOR_DEFAULT);

        let delegate_action = near_kit::DelegateAction {
            sender_id: unsigned_transaction.signer_id.clone(),
            receiver_id: unsigned_transaction.receiver_id.clone(),
            actions: unsigned_transaction
                .actions
                .into_iter()
                .map(near_kit::NonDelegateAction::try_from)
                .collect::<Result<_, _>>()
                .expect("Internal error: can not convert the action to non delegate action (delegate action can not be delegated again)."),
            nonce: unsigned_transaction.nonce,
            max_block_height,
            public_key: unsigned_transaction.public_key.clone(),
        };

        let signature = match sign_delegate_fn(
            &borsh::to_vec(&delegate_action)
                .wrap_err("Delegate action is not expected to fail on serialization")?,
            seed_phrase_hd_path_raw.clone(),
        ) {
            Ok(signature) => near_kit::Signature::ed25519_from_bytes(
                signature
                    .try_into()
                    .expect("Ledger ED25519 signature should be 64 bytes"),
            ),
            Err(NEARLedgerError::APDUExchangeError(msg)) if msg.contains(SW_BUFFER_OVERFLOW) => {
                return Err(color_eyre::Report::msg(ERR_OVERFLOW_MEMO));
            }
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "Error occurred while signing the transaction: {near_ledger_error:?}"
                )));
            }
        };
        let signed_delegate_action = delegate_action.sign(signature);

        return Ok(super::SubmitContext {
            network_config: previous_context.network_config.clone(),
            global_context: previous_context.global_context.clone(),
            signed_transaction_or_signed_delegate_action: signed_delegate_action.into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback
                .clone(),
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback
                .clone(),
            on_sending_delegate_action_callback: previous_context
                .on_sending_delegate_action_callback
                .clone(),
        });
    }

    let signature = match sign_tx_fn(
        &borsh::to_vec(&unsigned_transaction)
            .wrap_err("Transaction is not expected to fail on serialization")?,
        seed_phrase_hd_path_raw.clone(),
    ) {
        Ok(signature) => near_kit::Signature::ed25519_from_bytes(
            signature
                .try_into()
                .expect("Ledger ED25519 signature should be 64 bytes"),
        ),
        Err(NEARLedgerError::APDUExchangeError(msg)) if msg.contains(SW_BUFFER_OVERFLOW) => {
            return Err(color_eyre::Report::msg(ERR_OVERFLOW_MEMO));
        }
        Err(near_ledger_error) => {
            return Err(color_eyre::Report::msg(format!(
                "Error occurred while signing the transaction: {near_ledger_error:?}"
            )));
        }
    };

    let mut signed_transaction = unsigned_transaction.complete(signature.clone());

    tracing::info!(
        parent: &tracing::Span::none(),
        "Your transaction was signed successfully.{}",
        crate::common::indent_payload(&format!(
            "\nPublic key: {}\nSignature:  {}\n ",
            signer_public_key,
            signature
        ))
    );

    (previous_context.on_after_signing_callback)(
        &mut signed_transaction,
        &previous_context.network_config,
    )?;

    Ok(super::SubmitContext {
        network_config: previous_context.network_config.clone(),
        global_context: previous_context.global_context.clone(),
        signed_transaction_or_signed_delegate_action: signed_transaction.into(),
        on_before_sending_transaction_callback: previous_context
            .on_before_sending_transaction_callback
            .clone(),
        on_after_sending_transaction_callback: previous_context
            .on_after_sending_transaction_callback
            .clone(),
        on_sending_delegate_action_callback: previous_context
            .on_sending_delegate_action_callback
            .clone(),
    })
}

#[tracing::instrument(
    name = "Signing the transaction with Ledger device via USB. Follow the instructions on the ledger ...",
    skip_all
)]
fn sign_transaction_with_usb(
    previous_context: &crate::commands::TransactionContext,
    signer_public_key: &crate::types::public_key::PublicKey,
    seed_phrase_hd_path: &crate::types::slip10::BIP32Path,
    nonce: Option<u64>,
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    block_height: Option<u64>,
    meta_transaction_valid_for: Option<u64>,
) -> color_eyre::eyre::Result<UsbConnectionContext> {
    tracing::info!(target: "near_teach_me", "Signing the transaction with Ledger device via USB. Follow the instructions on the ledger ...");

    Ok(UsbConnectionContext(sign_transaction_with_ledger(
        previous_context,
        signer_public_key,
        seed_phrase_hd_path,
        nonce,
        block_hash,
        block_height,
        meta_transaction_valid_for,
        near_ledger::sign_transaction,
        near_ledger::sign_message_nep366_delegate_action,
    )?))
}

#[cfg(feature = "ledger-ble")]
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignLedgerContext)]
#[interactive_clap(output_context = BluetoothConnectionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct BluetoothConnection {
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[cfg(feature = "ledger-ble")]
#[derive(Clone)]
pub struct BluetoothConnectionContext(super::SubmitContext);

#[cfg(feature = "ledger-ble")]
impl From<BluetoothConnectionContext> for super::SubmitContext {
    fn from(item: BluetoothConnectionContext) -> Self {
        item.0
    }
}

#[cfg(feature = "ledger-ble")]
impl interactive_clap::FromCli for BluetoothConnection {
    type FromCliContext = SignLedgerContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<BluetoothConnection as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        let seed_phrase_hd_path = context.seed_phrase_hd_path.clone();

        let (ble_session, public_key) =
            match ble_helpers::BleSession::connect_open_and_get_public_key(
                seed_phrase_hd_path.clone().into(),
            ) {
                Ok(pair) => pair,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        let signer_public_key: crate::types::public_key::PublicKey =
            near_kit::PublicKey::ed25519_from_bytes(public_key.to_bytes()).into();

        let output_context = match sign_transaction_with_ble(
            &context.transaction_context,
            &signer_public_key,
            &seed_phrase_hd_path,
            &ble_session,
            context.nonce,
            context.block_hash,
            context.block_height,
            context.meta_transaction_valid_for,
        ) {
            Ok(ctx) => ctx,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(submit) => {
                clap_variant.submit = Some(submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_submit) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_submit, err) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

#[cfg(feature = "ledger-ble")]
#[tracing::instrument(
    name = "Signing the transaction with Ledger device via Bluetooth. Follow the instructions on the ledger ...",
    skip_all
)]
#[allow(clippy::too_many_arguments)]
fn sign_transaction_with_ble(
    previous_context: &crate::commands::TransactionContext,
    signer_public_key: &crate::types::public_key::PublicKey,
    seed_phrase_hd_path: &crate::types::slip10::BIP32Path,
    ble_session: &ble_helpers::BleSession,
    nonce: Option<u64>,
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    block_height: Option<u64>,
    meta_transaction_valid_for: Option<u64>,
) -> color_eyre::eyre::Result<BluetoothConnectionContext> {
    tracing::info!(target: "near_teach_me", "Signing the transaction with Ledger device via Bluetooth. Follow the instructions on the ledger ...");

    Ok(BluetoothConnectionContext(sign_transaction_with_ledger(
        previous_context,
        signer_public_key,
        seed_phrase_hd_path,
        nonce,
        block_hash,
        block_height,
        meta_transaction_valid_for,
        |data, path| ble_session.sign_transaction(data, path),
        |data, path| ble_session.sign_message_nep366_delegate_action(data, path),
    )?))
}

impl SignLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        input_seed_phrase_hd_path()
    }

    fn input_nonce(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter a nonce for the access key:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_hash(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::crypto_hash::CryptoHash>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<crate::types::crypto_hash::CryptoHash>::new(
                    "Enter recent block hash:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_height(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter recent block height:").prompt()?,
            ));
        }
        Ok(None)
    }
}

pub fn input_seed_phrase_hd_path()
-> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
    Ok(Some(
        CustomType::new("Enter seed phrase HD Path (if you're not sure, leave blank for default):")
            .with_starting_input("44'/397'/0'/0'/1'")
            .prompt()?,
    ))
}
