//! Helpers for converting between omni-transaction and near-primitives types
//! 
//! omni-transaction types are borsh-compatible with near-primitives types,
//! so we use borsh serialization/deserialization for conversion.

use color_eyre::eyre::{Context, Result};

/// Convert an omni-transaction NearTransaction to a near-primitives Transaction
pub fn omni_transaction_to_near_primitives(
    transaction: &omni_transaction::near::NearTransaction,
) -> Result<near_primitives::transaction::Transaction> {
    let serialized = borsh::to_vec(transaction)
        .wrap_err("Failed to serialize omni-transaction NearTransaction")?;
    
    let v0: near_primitives::transaction::TransactionV0 = borsh::from_slice(&serialized)
        .wrap_err("Failed to deserialize as near-primitives TransactionV0")?;
    
    Ok(near_primitives::transaction::Transaction::V0(v0))
}

/// Convert a near-primitives Transaction to an omni-transaction NearTransaction
pub fn near_primitives_transaction_to_omni(
    transaction: &near_primitives::transaction::Transaction,
) -> Result<omni_transaction::near::NearTransaction> {
    let serialized = borsh::to_vec(transaction)
        .wrap_err("Failed to serialize near-primitives Transaction")?;
    
    borsh::from_slice(&serialized)
        .wrap_err("Failed to deserialize as omni-transaction NearTransaction")
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_transaction::near::types::U64;

    #[test]
    fn test_transaction_roundtrip() {
        // Create an omni-transaction
        let omni_tx = omni_transaction::near::NearTransaction {
            signer_id: "alice.near".parse().unwrap(),
            signer_public_key: omni_transaction::near::types::PublicKey::ED25519(
                omni_transaction::near::types::ED25519PublicKey([0; 32])
            ),
            nonce: U64(1),
            receiver_id: "bob.near".parse().unwrap(),
            block_hash: omni_transaction::near::types::BlockHash([0; 32]),
            actions: vec![],
        };

        // Convert to near-primitives and back
        let near_tx = omni_transaction_to_near_primitives(&omni_tx).unwrap();
        let omni_tx_2 = near_primitives_transaction_to_omni(&near_tx).unwrap();

        // Compare serialized forms
        let omni_serialized = borsh::to_vec(&omni_tx).unwrap();
        let omni_serialized_2 = borsh::to_vec(&omni_tx_2).unwrap();
        
        assert_eq!(omni_serialized, omni_serialized_2);
    }
}
