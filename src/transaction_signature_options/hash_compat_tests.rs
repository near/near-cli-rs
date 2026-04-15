/// Gate commit G1: Verify that near-primitives' Transaction::V0(TransactionV0) borsh-serializes
/// byte-for-byte identically to near-kit's flat Transaction struct.
///
/// If this test passes, external signers (Ledger, MPC) keep working after we swap the types.
#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;

    // Fixed ed25519 public key bytes (deterministic, valid curve point).
    // This is the public key for the all-ones secret key.
    fn fixed_ed25519_public_key_bytes() -> [u8; 32] {
        // Use ed25519-dalek to derive the public key from a fixed secret key.
        let secret_bytes = [1u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        verifying_key.to_bytes()
    }

    fn fixed_block_hash() -> [u8; 32] {
        [0x11u8; 32]
    }

    /// Test 1: Simple transfer — single Transfer action.
    #[test]
    fn hash_compat_transfer() {
        let pk_bytes = fixed_ed25519_public_key_bytes();
        let block_hash_bytes = fixed_block_hash();

        // --- near-primitives side ---
        let np_pk =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey(pk_bytes));
        let np_block_hash = near_primitives::hash::CryptoHash(block_hash_bytes);

        let np_tx = near_primitives::transaction::Transaction::V0(
            near_primitives::transaction::TransactionV0 {
                signer_id: "alice.testnet".parse().unwrap(),
                public_key: np_pk,
                nonce: 42,
                receiver_id: "bob.testnet".parse().unwrap(),
                block_hash: np_block_hash,
                actions: vec![near_primitives::transaction::Action::Transfer(
                    near_primitives::transaction::TransferAction {
                        deposit: near_token::NearToken::from_near(1),
                    },
                )],
            },
        );

        // --- near-kit side ---
        let nk_pk = near_kit::PublicKey::ed25519_from_bytes(pk_bytes);
        let nk_block_hash = near_kit::CryptoHash::from_bytes(block_hash_bytes);

        let nk_tx = near_kit::Transaction::new(
            "alice.testnet".parse().unwrap(),
            nk_pk,
            42,
            "bob.testnet".parse().unwrap(),
            nk_block_hash,
            vec![near_kit::Action::Transfer(near_kit::TransferAction {
                deposit: near_kit::NearToken::from_near(1),
            })],
        );

        // Byte-for-byte serialization equality.
        let np_bytes = borsh::to_vec(&np_tx).unwrap();
        let nk_bytes = borsh::to_vec(&nk_tx).unwrap();

        assert_eq!(
            np_bytes, nk_bytes,
            "near-primitives Transaction::V0 and near-kit Transaction borsh output differ!\n\
             np hex: {}\nnk hex: {}",
            hex::encode(&np_bytes),
            hex::encode(&nk_bytes),
        );

        // Hash equality (both SHA-256 of the borsh bytes).
        let (np_hash, _) = np_tx.get_hash_and_size();
        let nk_hash = nk_tx.get_hash();
        assert_eq!(
            np_hash.0,
            *nk_hash.as_bytes(),
            "Transaction hashes differ!",
        );
    }

    /// Test 2: Richer action set — CreateAccount + Transfer + AddKey(FullAccess).
    #[test]
    fn hash_compat_rich_actions() {
        let pk_bytes = fixed_ed25519_public_key_bytes();
        let block_hash_bytes = fixed_block_hash();

        // --- near-primitives side ---
        let np_pk =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey(pk_bytes));
        let np_block_hash = near_primitives::hash::CryptoHash(block_hash_bytes);

        let np_tx = near_primitives::transaction::Transaction::V0(
            near_primitives::transaction::TransactionV0 {
                signer_id: "alice.testnet".parse().unwrap(),
                public_key: np_pk.clone(),
                nonce: 42,
                receiver_id: "bob.testnet".parse().unwrap(),
                block_hash: np_block_hash,
                actions: vec![
                    near_primitives::transaction::Action::CreateAccount(
                        near_primitives::transaction::CreateAccountAction {},
                    ),
                    near_primitives::transaction::Action::Transfer(
                        near_primitives::transaction::TransferAction {
                            deposit: near_token::NearToken::from_near(1),
                        },
                    ),
                    near_primitives::transaction::Action::AddKey(Box::new(
                        near_primitives::transaction::AddKeyAction {
                            public_key: np_pk.clone(),
                            access_key: near_primitives::account::AccessKey {
                                nonce: 0,
                                permission:
                                    near_primitives::account::AccessKeyPermission::FullAccess,
                            },
                        },
                    )),
                ],
            },
        );

        // --- near-kit side ---
        let nk_pk = near_kit::PublicKey::ed25519_from_bytes(pk_bytes);
        let nk_block_hash = near_kit::CryptoHash::from_bytes(block_hash_bytes);

        let nk_tx = near_kit::Transaction::new(
            "alice.testnet".parse().unwrap(),
            nk_pk.clone(),
            42,
            "bob.testnet".parse().unwrap(),
            nk_block_hash,
            vec![
                near_kit::Action::CreateAccount(near_kit::CreateAccountAction),
                near_kit::Action::Transfer(near_kit::TransferAction {
                    deposit: near_kit::NearToken::from_near(1),
                }),
                near_kit::Action::AddKey(near_kit::AddKeyAction {
                    public_key: nk_pk.clone(),
                    access_key: near_kit::AccessKey::full_access(),
                }),
            ],
        );

        // Byte-for-byte serialization equality.
        let np_bytes = borsh::to_vec(&np_tx).unwrap();
        let nk_bytes = borsh::to_vec(&nk_tx).unwrap();

        assert_eq!(
            np_bytes, nk_bytes,
            "near-primitives Transaction::V0 and near-kit Transaction borsh output differ (rich actions)!\n\
             np hex: {}\nnk hex: {}",
            hex::encode(&np_bytes),
            hex::encode(&nk_bytes),
        );

        // Hash equality.
        let (np_hash, _) = np_tx.get_hash_and_size();
        let nk_hash = nk_tx.get_hash();
        assert_eq!(
            np_hash.0,
            *nk_hash.as_bytes(),
            "Transaction hashes differ (rich actions)!",
        );
    }
}
