extern crate commoncrypto;
extern crate hex;

use commoncrypto::pbkdf2::{pbkdf2, CCPseudoRandomAlgorithm};
use hex::ToHex;

#[test]
fn derive_pbkdf2() {
    let derived = pbkdf2(b"password",
                         b"salt",
                         CCPseudoRandomAlgorithm::kCCPRFHmacAlgSHA1,
                         1,
                         20)
        .unwrap();
    assert_eq!("0c60c80f961f0e71f3a9b524af6012062fe037a6", derived.to_hex());
}
