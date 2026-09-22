use near_cli_rs::common::{GeneratedKeyPair, SignatureScheme};
use near_cli_rs::config::Config;
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

    // The printed strings must parse back into near_crypto types of the
    // post-quantum key type, and the secret key must derive the public key.
    let parsed_public = near_crypto::PublicKey::from_str(public_key).unwrap();
    let parsed_secret = near_crypto::SecretKey::from_str(private_key).unwrap();
    // `KeyType` intentionally does not implement `PartialEq` in 2.13, so we
    // match on the variant instead of comparing with `assert_eq!`.
    assert!(matches!(
        parsed_public.key_type(),
        near_crypto::KeyType::MLDSA65
    ));
    assert!(matches!(
        parsed_secret.key_type(),
        near_crypto::KeyType::MLDSA65
    ));
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
    assert!(near_crypto::PublicKey::from_str(&properties.public_key_str).is_ok());
}

// The keychain / legacy-keychain identifier a key is *saved* under must equal
// the string the RPC access-key list reports (a `near_crypto::PublicKeyHandle`),
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

    // It must be exactly what the RPC access-key list would report for this key
    // (`PublicKeyHandle`), so a saved key can be found again for signing.
    let public_key = key_pair.public_key().unwrap();
    let handle = near_crypto::PublicKeyHandle::from(&public_key).to_string();
    assert_eq!(key_id, handle);
}

#[test]
fn legacy_keychain_stores_typed_public_keys_under_canonical_ids() {
    let credentials_home_dir = tempfile::tempdir().unwrap();
    let network_config = Config::default()
        .network_connection
        .get("testnet")
        .unwrap()
        .clone();

    for (account_id, key_type) in [
        ("ml-dsa.testnet", near_crypto::KeyType::MLDSA65),
        ("ed25519.testnet", near_crypto::KeyType::ED25519),
        ("secp256k1.testnet", near_crypto::KeyType::SECP256K1),
    ] {
        let private_key = near_crypto::SecretKey::from_random(key_type);
        let public_key = private_key.public_key();
        let keychain_key_id = near_crypto::PublicKeyHandle::from(&public_key).to_string();
        let keychain_json = serde_json::to_string(&serde_json::json!({
            "public_key": public_key,
            "private_key": private_key,
        }))
        .unwrap();
        near_cli_rs::common::save_access_key_to_legacy_keychain(
            network_config.clone(),
            credentials_home_dir.path().to_path_buf(),
            &keychain_json,
            &public_key,
            account_id,
        )
        .unwrap();

        let signer_key_dir = credentials_home_dir.path().join("testnet").join(account_id);
        let saved_key_files = signer_key_dir
            .read_dir()
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();

        assert_eq!(
            saved_key_files,
            vec![std::ffi::OsString::from(format!(
                "{}.json",
                keychain_key_id.replace(':', "_")
            ))]
        );
        assert!(saved_key_files[0].len() < 255);
        assert_eq!(
            std::fs::read_to_string(signer_key_dir.join(&saved_key_files[0])).unwrap(),
            keychain_json
        );
        assert_eq!(
            std::fs::read_to_string(
                credentials_home_dir
                    .path()
                    .join("testnet")
                    .join(format!("{account_id}.json"))
            )
            .unwrap(),
            keychain_json
        );
    }
}
