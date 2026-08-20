use near_cli_rs::common::{GeneratedKeyPair, SignatureScheme};
use std::str::FromStr;

#[test]
fn ml_dsa_65_keypair_roundtrips() {
    let key_pair = GeneratedKeyPair::generate(&SignatureScheme::MlDsa65).unwrap();
    let GeneratedKeyPair::MlDsa65 {
        public_key,
        private_key,
    } = &key_pair
    else {
        panic!("expected an ML-DSA-65 key pair");
    };
    assert!(public_key.starts_with("ml-dsa-65:"), "{public_key}");
    assert!(private_key.starts_with("ml-dsa-65:"), "{private_key}");

    // The printed strings must parse back into near-kit types of the
    // post-quantum key type, and the secret key must derive the public key.
    let parsed_public = near_kit::PublicKey::from_str(public_key).unwrap();
    let parsed_secret = near_kit::SecretKey::from_str(private_key).unwrap();
    assert!(matches!(&parsed_public, near_kit::PublicKey::MlDsa65(_)));
    assert!(matches!(&parsed_secret, near_kit::SecretKey::MlDsa65(_)));
    assert_eq!(parsed_secret.public_key(), parsed_public);

    // A signature produced by the secret key must verify under the public key.
    let message = b"post-quantum near-cli-rs";
    let signature = parsed_secret.sign(message);
    assert!(signature.verify(message, &parsed_public));
}

#[test]
fn ed25519_remains_the_classic_default() {
    let key_pair = GeneratedKeyPair::generate(&SignatureScheme::Ed25519).unwrap();
    let GeneratedKeyPair::Ed25519(properties) = &key_pair else {
        panic!("expected an Ed25519 key pair");
    };
    assert!(properties.public_key_str.starts_with("ed25519:"));
    assert!(near_kit::PublicKey::from_str(&properties.public_key_str).is_ok());
}

// The keychain / legacy-keychain identifier a key is *saved* under must equal
// the string the RPC access-key list reports (an ML-DSA-65 hash handle),
// because that is what the signers look the key up by. For ed25519 that is the
// full public key; for ML-DSA-65 it is the short `ml-dsa-65-hash:...` handle,
// never the ~1952-byte full key.
#[test]
fn ed25519_keychain_id_is_the_full_public_key() {
    let key_pair = GeneratedKeyPair::generate(&SignatureScheme::Ed25519).unwrap();
    assert_eq!(
        key_pair.keychain_key_id().unwrap(),
        key_pair.public_key_str()
    );
}

#[test]
fn ml_dsa_65_keychain_id_is_the_on_chain_handle() {
    let key_pair = GeneratedKeyPair::generate(&SignatureScheme::MlDsa65).unwrap();
    let key_id = key_pair.keychain_key_id().unwrap();

    // The saved id is the on-trie handle, not the full key.
    assert!(key_id.starts_with("ml-dsa-65-hash:"), "{key_id}");
    assert_ne!(key_id, key_pair.public_key_str());
    // Bounded length (well under any filesystem name limit), unlike the ~2.6KB
    // full ML-DSA-65 key string.
    assert!(key_id.len() < 80, "{} chars", key_id.len());

    // It must be exactly what the RPC access-key list would report for this key,
    // so a saved key can be found again for signing.
    let public_key = key_pair.public_key().unwrap();
    let handle = public_key.to_ml_dsa65_hash().unwrap().to_string();
    assert_eq!(key_id, handle);
}
