use near_cli_rs::types::{
    signed_delegate_action::SignedDelegateActionAsBase64,
    signed_transaction::SignedTransactionAsBase64, transaction::TransactionAsBase64,
};
use near_primitives::{
    action::{
        Action, TransferAction,
        delegate::{DelegateAction, SignedDelegateAction},
    },
    hash::CryptoHash,
    transaction::{
        NonceMode, SignedTransaction, Transaction, TransactionNonce, TransactionV0, TransactionV1,
    },
};

const UNSIGNED_V0: &str = include_str!("fixtures/transaction_compatibility/unsigned_v0.base64");
const SIGNED_V0: &str = include_str!("fixtures/transaction_compatibility/signed_v0.base64");
const SIGNED_V1: &str = include_str!("fixtures/transaction_compatibility/signed_v1.base64");
const DELEGATE: &str = include_str!("fixtures/transaction_compatibility/signed_delegate.base64");

fn transaction_v0() -> TransactionV0 {
    TransactionV0 {
        signer_id: "alice.near".parse().unwrap(),
        public_key: near_crypto::PublicKey::ED25519([7; 32].into()),
        nonce: 42,
        receiver_id: "bob.near".parse().unwrap(),
        block_hash: CryptoHash([9; 32]),
        actions: vec![Action::Transfer(TransferAction {
            deposit: near_token::NearToken::from_yoctonear(123),
        })],
    }
}

fn signature() -> near_crypto::Signature {
    near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &[11; 64]).unwrap()
}

#[test]
fn unsigned_v0_is_tagless() {
    let expected = transaction_v0();
    let decoded: TransactionAsBase64 = UNSIGNED_V0.trim().parse().unwrap();
    assert_eq!(decoded.inner, expected);
    assert_eq!(
        TransactionAsBase64::from(expected).to_string(),
        UNSIGNED_V0.trim()
    );
}

#[test]
fn signed_v0_and_v1_preserve_the_wire_format() {
    let v0 = transaction_v0();
    let v1 = TransactionV1 {
        signer_id: v0.signer_id.clone(),
        public_key: v0.public_key.clone(),
        nonce: TransactionNonce::GasKeyNonce {
            nonce: 42,
            nonce_index: 3,
        },
        receiver_id: v0.receiver_id.clone(),
        block_hash: v0.block_hash,
        actions: v0.actions.clone(),
        nonce_mode: NonceMode::Strict,
    };
    for (fixture, transaction) in [
        (SIGNED_V0, Transaction::V0(v0)),
        (SIGNED_V1, Transaction::V1(v1)),
    ] {
        let decoded: SignedTransactionAsBase64 = fixture.trim().parse().unwrap();
        let expected = SignedTransaction::new(signature(), transaction);
        assert_eq!(decoded.inner, expected);
        assert_eq!(
            SignedTransactionAsBase64::from(expected).to_string(),
            fixture.trim()
        );
    }
}

#[test]
fn signed_delegate_preserves_text_and_json_encoding() {
    let v0 = transaction_v0();
    let expected = SignedDelegateAction {
        delegate_action: DelegateAction {
            sender_id: v0.signer_id,
            receiver_id: v0.receiver_id,
            actions: v0
                .actions
                .into_iter()
                .map(|action| action.try_into().unwrap())
                .collect(),
            nonce: 42,
            max_block_height: 1000,
            public_key: v0.public_key,
        },
        signature: signature(),
    };
    let decoded: SignedDelegateActionAsBase64 = DELEGATE.trim().parse().unwrap();
    assert_eq!(SignedDelegateAction::from(decoded), expected);
    let encoded = SignedDelegateActionAsBase64::from(expected.clone());
    assert_eq!(encoded.to_string(), DELEGATE.trim());
    let json = serde_json::to_string(DELEGATE.trim()).unwrap();
    assert_eq!(serde_json::to_string(&encoded).unwrap(), json);
    let decoded: SignedDelegateActionAsBase64 = serde_json::from_str(&json).unwrap();
    assert_eq!(SignedDelegateAction::from(decoded), expected);
}

#[test]
fn parsers_reject_trailing_bytes() {
    fn with_garbage(fixture: &str) -> String {
        let mut bytes = near_primitives::serialize::from_base64(fixture.trim()).unwrap();
        bytes.push(0);
        near_primitives::serialize::to_base64(&bytes)
    }
    assert!(
        with_garbage(UNSIGNED_V0)
            .parse::<TransactionAsBase64>()
            .is_err()
    );
    for fixture in [SIGNED_V0, SIGNED_V1] {
        assert!(
            with_garbage(fixture)
                .parse::<SignedTransactionAsBase64>()
                .is_err()
        );
    }
    let delegate = with_garbage(DELEGATE);
    assert!(delegate.parse::<SignedDelegateActionAsBase64>().is_err());
    assert!(
        serde_json::from_value::<SignedDelegateActionAsBase64>(serde_json::json!(delegate))
            .is_err()
    );
}

#[test]
fn parsers_reject_invalid_base64() {
    for invalid in ["!", "AA", "AB=="] {
        assert!(
            invalid
                .parse::<TransactionAsBase64>()
                .unwrap_err()
                .contains("base64")
        );
        assert!(
            invalid
                .parse::<SignedTransactionAsBase64>()
                .unwrap_err()
                .contains("base64")
        );
        assert!(
            invalid
                .parse::<SignedDelegateActionAsBase64>()
                .unwrap_err()
                .contains("base64")
        );
    }
}
