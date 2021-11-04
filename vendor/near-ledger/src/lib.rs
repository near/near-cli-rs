//! NEAR <-> Ledger transport
//!
//! Provides a set of commands that can be executed to communicate with NEAR App installed on Ledger device:
//! - Read PublicKey from Ledger device by HD Path
//! - Sign a Transaction
use ledger::TransportNativeHID;
use ledger_apdu::map_apdu_error_description;
use ledger_transport::errors::TransportError;
use ledger_transport::{APDUCommand, APDUTransport};

const CLA: u8 = 0x80; // Instruction class
const INS_GET_PUBLIC_KEY: u8 = 4; // Instruction code to get public key
const INS_GET_VERSION: u8 = 6; // Instruction code to get app version from the Ledger
const INS_SIGN_TRANSACTION: u8 = 2; // Instruction code to sign a transaction on the Ledger
const NETWORK_ID: u8 = 'W' as u8; // Instruction parameter 2
const RETURN_CODE_OK: u16 = 36864; // APDUAnswer.retcode which means success from Ledger
const CHUNK_SIZE: usize = 128; // Chunk size to be sent to Ledger

/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type BorshSerializedUnsignedTransaction = Vec<u8>;
/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type NEARLedgerAppVersion = Vec<u8>;
/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type SignatureBytes = Vec<u8>;

#[derive(Debug)]
pub enum NEARLedgerError {
    /// Self-explanatory one
    DeviceNotFound,
    /// Error occurred while exchanging with Ledger device
    APDUExchangeError(String),
    /// Error with transport
    APDUTransportError(TransportError),
}

/// Converts BIP32Path into bytes (`Vec<u8>`)
fn hd_path_to_bytes(hd_path: &slip10::BIP32Path) -> Vec<u8> {
    (0..hd_path.depth())
        .map(|index| {
            let value = *hd_path.index(index).unwrap();
            value.to_be_bytes()
        })
        .flatten()
        .collect::<Vec<u8>>()
}

/// Get the version of NEAR App installed on Ledger
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `NEARLedgerAppVersion` (just a `Vec<u8>` for now, where first value is a major version, second is a minor and the last is the path)
///  and whose `Err` value is a `NEARLedgerError` containing an error which occurred.
pub async fn get_version() -> Result<NEARLedgerAppVersion, NEARLedgerError> {
    //! Something
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = match TransportNativeHID::new() {
        Ok(transport) => APDUTransport::new(transport),
        // TODO: refactor this
        // https://github.com/Zondax/ledger-rs/issues/65
        Err(_err) => return Err(NEARLedgerError::DeviceNotFound),
    };

    match transport
        .exchange(&APDUCommand {
            cla: CLA,
            ins: INS_GET_VERSION,
            p1: 0, // Instruction parameter 1 (offset)
            p2: 0,
            data: vec![],
        })
        .await
    {
        Ok(response) => {
            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            if response.retcode == RETURN_CODE_OK {
                return Ok(response.data);
            } else {
                let error_string = map_apdu_error_description(response.retcode).to_string();
                return Err(NEARLedgerError::APDUExchangeError(error_string));
            }
        }
        Err(err) => return Err(NEARLedgerError::APDUTransportError(err)),
    };
}

/// Gets PublicKey from the Ledger on the given `hd_path`
///
/// # Inputs
/// * `hd_path` - seed phrase hd path `slip10::BIP32Path` for which PublicKey to look
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `ed25519_dalek::PublicKey` and whose `Err` value is a
///   `NEARLedgerError` containing an error which
///   occured.
///
/// # Examples
///
/// ```
/// use near_ledger::get_public_key;
/// use slip10::BIP32Path;
///
/// # asyn fn main() {
/// let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
/// let public_key = match get_public_key(hd_path)
///    .await
///    .map_err(|near_ledger_error| {
///        panic!(
///            "An error occured while getting PublicKey from Ledger device: {:?}",
///             near_ledger_error,
///        )
///    })?;
/// # }
/// ```
///
/// # Trick
///
/// To convert the answer into `near_crypto::PublicKey` do:
///
/// ```
/// near_crypto::PublicKey::ED25519(
///     near_crypto::ED25519PublicKey::from(
///         public_key.to_bytes(),
///     )
/// )
/// ```
pub async fn get_public_key(
    hd_path: slip10::BIP32Path,
) -> Result<ed25519_dalek::PublicKey, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = match TransportNativeHID::new() {
        Ok(transport) => APDUTransport::new(transport),
        // TODO: refactor this
        // https://github.com/Zondax/ledger-rs/issues/65
        Err(_err) => return Err(NEARLedgerError::DeviceNotFound),
    };

    // hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&hd_path);

    match transport
        .exchange(&APDUCommand {
            cla: CLA,
            ins: INS_GET_PUBLIC_KEY,
            p1: 0, // Instruction parameter 1 (offset)
            p2: NETWORK_ID,
            data: hd_path_bytes,
        })
        .await
    {
        Ok(response) => {
            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            if response.retcode == RETURN_CODE_OK {
                return Ok(ed25519_dalek::PublicKey::from_bytes(&response.data).unwrap());
            } else {
                let error_string = map_apdu_error_description(response.retcode).to_string();
                return Err(NEARLedgerError::APDUExchangeError(error_string));
            }
        }
        Err(err) => return Err(NEARLedgerError::APDUTransportError(err)),
    };
}

/// Sign the transaction. Transaction should be [borsh serialized](https://github.com/near/borsh-rs) `Vec<u8>`
///
/// # Inputs
/// * `unsigned_transaction_borsh_serializer` - unsigned transaction `near_primitives::transaction::Transaction`
/// which is serialized with `BorshSerializer` and basically is just `Vec<u8>`
/// * `seed_phrase_hd_path` - seed phrase hd path `slip10::BIP32Path` with which to sign
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `Signature` (bytes) and whose `Err` value is a
/// `NEARLedgerError` containing an error which occured.
///
/// # Examples
///
/// ```
/// use near_ledger::sign_transaction;
/// use borsh::BorshSerializer;
/// use slip10::BIP32Path;
///
/// # asyn fn main() {
/// let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
/// let borsh_transaction = near_unsigned_transaction.try_to_vec().unwrap();
/// let signature = match sign_transaction(borsh_transaction, hd_path)
///    .await
///    .map_err(|near_ledger_error| {
///        panic!(
///            "An error occured while getting PublicKey from Ledger device: {:?}",
///             near_ledger_error,
///        )
///    })?;
/// # }
/// ```
///
/// # Trick
///
/// To convert the answer into `near_crypto::Signature` do:
///
/// ```
/// near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
///     .expect("Signature is not expected to fail on deserialization")
/// ```
pub async fn sign_transaction(
    unsigned_transaction_borsh_serializer: BorshSerializedUnsignedTransaction,
    seed_phrase_hd_path: slip10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = match TransportNativeHID::new() {
        Ok(transport) => APDUTransport::new(transport),
        // TODO: refactor this
        // https://github.com/Zondax/ledger-rs/issues/65
        Err(_err) => return Err(NEARLedgerError::DeviceNotFound),
    };

    // seed_phrase_hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&seed_phrase_hd_path);

    let mut data: Vec<u8> = vec![];
    data.extend(hd_path_bytes);
    data.extend(unsigned_transaction_borsh_serializer);

    let chunks = data.chunks(CHUNK_SIZE);
    let chunks_count = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        let is_last_chunk = chunks_count == i + 1;
        match transport
            .exchange(&APDUCommand {
                cla: CLA,
                ins: INS_SIGN_TRANSACTION,
                p1: if is_last_chunk { 0x80 } else { 0 }, // Instruction parameter 1 (offset)
                p2: NETWORK_ID,
                data: chunk.to_vec(),
            })
            .await
        {
            Ok(response) => {
                // Ok means we successfully exchanged with the Ledger
                // but doesn't mean our request succeeded
                // we need to check it based on `response.retcode`
                if response.retcode == RETURN_CODE_OK {
                    if is_last_chunk {
                        return Ok(response.data);
                    }
                } else {
                    let error_string = map_apdu_error_description(response.retcode).to_string();
                    return Err(NEARLedgerError::APDUExchangeError(error_string));
                }
            }
            Err(err) => return Err(NEARLedgerError::APDUTransportError(err)),
        };
    }
    Err(NEARLedgerError::APDUExchangeError(
        "Unable to process request".to_owned(),
    ))
}
