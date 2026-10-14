use std::collections::{BTreeMap, BTreeSet};

use near_crypto::{MlDsa65PublicKeyHandle, PublicKeyHandle};
use near_primitives::action::{GlobalContractIdentifier, UniversalStateInitAction};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::Action;
use near_primitives::universal_state_init::{
    RawStateInit, UniversalStateInit, UniversalStateInitV1,
};

fn preview(raw: RawStateInit) -> String {
    near_cli_rs::common::print_unsigned_transaction(
        &near_cli_rs::commands::PrepopulatedTransaction {
            signer_id: "signer.near".parse().unwrap(),
            receiver_id: "receiver.near".parse().unwrap(),
            actions: vec![Action::UniversalStateInit(Box::new(
                UniversalStateInitAction {
                    state_init: raw,
                    deposit: near_token::NearToken::from_near(1),
                },
            ))],
        },
    )
}

#[test]
fn canonical_universal_state_init_preview_keeps_known_account_ids() {
    let key_only = UniversalStateInit::V1(UniversalStateInitV1 {
        code: None,
        data: BTreeMap::new(),
        access_keys: BTreeSet::from([PublicKeyHandle::MlDsa65(MlDsa65PublicKeyHandle([0x11; 32]))]),
    });
    let contract = UniversalStateInit::V1(UniversalStateInitV1 {
        code: Some(GlobalContractIdentifier::CodeHash(CryptoHash([0x22; 32]))),
        data: BTreeMap::from([(b"key".to_vec(), b"value".to_vec())]),
        access_keys: BTreeSet::new(),
    });

    // nearcore's known-answer vectors pin the canonical account IDs.
    for (state_init, expected_id) in [
        (
            key_only,
            "0ux8te7g99f9kqzdtp9h4qnwt9aczpgayymmtbdc50w199rcw3at1g",
        ),
        (
            contract,
            "0uzvdgbyea2rd8ywx0kw3cg4vc0ez1x5fc2gyks4fdz9ae0xxvzan0",
        ),
    ] {
        let output = preview(state_init.to_raw());
        assert!(output.contains(&format!("create universal account <{expected_id}>:")));
        let json = serde_json::to_string_pretty(&state_init).unwrap();
        assert!(output.contains(&json.replace('\n', &format!("\n{:33}", ""))));
        assert!(!output.contains("invalid"));
    }
}

fn assert_invalid_preview(raw: RawStateInit) {
    let derived_id = near_primitives::utils::derive_universal_account_id(&raw);
    let output = preview(raw);
    assert!(output.contains("invalid universal state-init (no derived account ID):"));
    assert!(output.contains("invalid:"));
    assert!(output.contains("deposit"));
    assert!(!output.contains("create universal account"));
    assert!(!output.contains(derived_id.as_str()));
}

// Shared verbatim with near-sdk-rs/near-global-contracts tests/data/universal_state_init.json.
// Canonicality is a wire-format contract across implementations, not just within this decoder.
#[test]
fn shared_canonicality_vectors_control_address_presentation() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/universal_state_init.json")).unwrap();
    for vector in vectors["valid"].as_array().unwrap() {
        let raw = RawStateInit(hex::decode(vector["hex"].as_str().unwrap()).unwrap());
        let output = preview(raw);
        let expected_id = vector["account_id"].as_str().unwrap();
        assert!(
            output.contains(&format!("create universal account <{expected_id}>:")),
            "{}",
            vector["name"],
        );
        assert!(!output.contains("invalid"), "{}", vector["name"]);
    }
    for vector in vectors["invalid"].as_array().unwrap() {
        let raw = RawStateInit(hex::decode(vector["hex"].as_str().unwrap()).unwrap());
        assert_invalid_preview(raw);
    }
}
