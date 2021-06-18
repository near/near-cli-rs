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

#[derive(Debug)]
pub(crate) enum NEARLedgerError {
    DeviceNotFound,
    APDUExchangeError(String),
    APDUTransportError(TransportError),
}

/// Gets PublicKey from the Ledger on the given `hd_path`
pub(crate) async fn get_public_key(
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
    let hd_path_bytes: Vec<u8> = (0..hd_path.depth())
        .map(|index| {
            let value = *hd_path.index(index).unwrap();
            value.to_be_bytes()
        })
        .flatten()
        .collect::<Vec<u8>>();

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
