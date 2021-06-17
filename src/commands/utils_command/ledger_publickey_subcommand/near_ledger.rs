use hex_slice::AsHex;

use ledger::TransportNativeHID;
use ledger_apdu::map_apdu_error_description;
use ledger_transport::{APDUTransport, APDUCommand, APDUAnswer};
// use ledger_transport::apdu_transport_native::APDUTransport;

pub(crate) async fn get_public_key(hd_path: slip10::BIP32Path) {
    let transport = match TransportNativeHID::new() {
        Ok(transport) => APDUTransport::new(transport),
        Err(err) => panic!("{:?}", err),
    };
    eprintln!("{}", hd_path);
    let hd_path_bytes: Vec<u8> = (0..hd_path.depth())
        .map(|index| {
            let value = *hd_path.index(index).unwrap();
            value.to_be_bytes()
        })
        .flatten()
        .collect::<Vec<u8>>();
    eprintln!("{:x}", &hd_path_bytes.as_hex());
    match transport.exchange(
        &APDUCommand {
            cla: 0x80,
            ins: 4,
            p1: 0,
            p2: 'W' as u8,
            data: hd_path_bytes,
        }
    ).await {
        Ok(response) => {
            println!("{:?}", &response);
            println!("{}", map_apdu_error_description(response.retcode));
            println!("{}", response.data.len());
            println!(
                "{:?}",
                bs58::encode(
                    &ed25519_dalek::PublicKey::from_bytes(&response.data).unwrap()
                ).into_string()
            );
        },
        Err(err) => panic!("{:?}", err)
    };
}
