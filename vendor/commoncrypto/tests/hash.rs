extern crate commoncrypto;
extern crate hex;

use commoncrypto::hash::{CCDigestAlgorithm, Hasher};
use hex::ToHex;
use std::io::Write;

const TO_HASH: &'static str = "The quick brown fox jumps over the lazy dog";
const TO_HASH_MD5: &'static str = "9e107d9d372bb6826bd81d3542a419d6";

#[test]
fn md5_hasher() {
    let mut hasher = Hasher::new(CCDigestAlgorithm::kCCDigestMD5);
    assert!(hasher.write_all(TO_HASH.as_bytes()).is_ok());
    let result = hasher.finish();
    assert!(result.is_ok());
    assert_eq!(result.expect("Hash failed").to_hex(), TO_HASH_MD5)
}
