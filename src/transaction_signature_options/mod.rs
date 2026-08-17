use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod display;
pub mod save_to_file;
pub mod send;
pub mod sign_later;
pub mod sign_with_access_key_file;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
pub mod sign_with_legacy_keychain;
pub mod sign_with_mpc;
pub mod sign_with_private_key;
pub mod sign_with_seed_phrase;
pub mod submit_dao_proposal;

pub const META_TRANSACTION_VALID_FOR_DEFAULT: u64 = 1000;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing the transaction:
pub enum SignWith {
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign the transaction with a key saved in the secure keychain"
    ))]
    /// Sign the transaction with a key saved in keychain
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-legacy-keychain        - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)
    SignWithLegacyKeychain(self::sign_with_legacy_keychain::SignLegacyKeychain),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "sign-with-ledger                 - Sign the transaction with Ledger Nano device"
    ))]
    /// Sign the transaction with Ledger Nano device
    SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-with-plaintext-private-key  - Sign the transaction with a plaintext private key"
    ))]
    /// Sign the transaction with a plaintext private key
    SignWithPlaintextPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(
        message = "sign-with-access-key-file        - Sign the transaction using the account access key file (access-key-file.json)"
    ))]
    /// Sign the transaction using the account access key file (access-key-file.json)
    SignWithAccessKeyFile(self::sign_with_access_key_file::SignAccessKeyFile),
    #[strum_discriminants(strum(
        message = "sign-with-seed-phrase            - Sign the transaction using the seed phrase"
    ))]
    /// Sign the transaction using the seed phrase
    SignWithSeedPhrase(self::sign_with_seed_phrase::SignSeedPhrase),
    #[strum_discriminants(strum(
        message = "sign-with-mpc                    - Sign and send the transaction with MPC"
    ))]
    /// Sign and send the transaction with MPC
    SignWithMpc(crate::transaction_signature_options::sign_with_mpc::SignMpc),
    #[strum_discriminants(strum(
        message = "sign-later                       - Prepare an unsigned transaction to sign it later"
    ))]
    /// Prepare unsigned transaction to sign it later
    SignLater(self::sign_later::SignLater),
    #[strum_discriminants(strum(
        message = "submit-as-dao-proposal           - Convert current transaction to DAO proposal"
    ))]
    /// Prepare transaction as dao proposal
    SubmitAsDaoProposal(self::submit_dao_proposal::DaoProposal),
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to proceed?
pub enum Submit {
    #[strum_discriminants(strum(
        message = "send             - Send the transaction to the network"
    ))]
    /// Send the transaction to the network
    Send(self::send::Send),
    #[strum_discriminants(strum(
        message = "save-to-file     - Save the signed transaction to file (if you want to send it later)"
    ))]
    /// Save the signed transaction to file (if you want to send it later)
    SaveToFile(self::save_to_file::SaveToFile),
    #[strum_discriminants(strum(
        message = "display          - Print the signed transaction to terminal (if you want to send it later)"
    ))]
    /// Print the signed transaction to terminal (if you want to send it later)
    Display(self::display::Display),
}

#[derive(Debug, Deserialize)]
pub struct AccountKeyPair {
    pub public_key: near_crypto::PublicKey,
    pub private_key: near_crypto::SecretKey,
}

pub type OnBeforeSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &SignedTransactionOrSignedDelegateAction,
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<String>,
>;

pub type OnAfterSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &near_primitives::views::FinalExecutionOutcomeView,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;

pub type OnSendingDelegateActionCallback = std::sync::Arc<
    dyn Fn(
        near_primitives::action::delegate::SignedDelegateAction,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;

#[derive(Clone)]
pub struct SubmitContext {
    pub network_config: crate::config::NetworkConfig,
    pub global_context: crate::GlobalContext,
    pub signed_transaction_or_signed_delegate_action: SignedTransactionOrSignedDelegateAction,
    pub on_before_sending_transaction_callback: OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback: OnAfterSendingTransactionCallback,
    pub on_sending_delegate_action_callback: Option<OnSendingDelegateActionCallback>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SignedTransactionOrSignedDelegateAction {
    SignedTransaction(near_primitives::transaction::SignedTransaction),
    /// NEP-366 meta-transaction, signed with an ordinary access key.
    SignedDelegateAction(near_primitives::action::delegate::SignedDelegateAction),
    /// NEP-611 meta-transaction, signed with a gas key (its nonce carries the
    /// parallel-nonce index that a V1 `SignedDelegateAction` cannot encode).
    SignedDelegateActionV2(near_primitives::action::delegate::VersionedSignedDelegateAction),
}

impl From<near_primitives::transaction::SignedTransaction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(signed_transaction: near_primitives::transaction::SignedTransaction) -> Self {
        Self::SignedTransaction(signed_transaction)
    }
}

impl From<near_primitives::action::delegate::SignedDelegateAction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(
        signed_delegate_action: near_primitives::action::delegate::SignedDelegateAction,
    ) -> Self {
        Self::SignedDelegateAction(signed_delegate_action)
    }
}

impl From<near_primitives::action::delegate::VersionedSignedDelegateAction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(
        signed_delegate_action: near_primitives::action::delegate::VersionedSignedDelegateAction,
    ) -> Self {
        Self::SignedDelegateActionV2(signed_delegate_action)
    }
}

/// A gas key cannot sign a meta-transaction on Ledger yet: the Ledger app only
/// implements NEP-366 delegate-action signing (`sign_message_nep366_delegate_action`),
/// which produces a V1 `DelegateAction` that has no room for the gas key's nonce
/// index. Carrying it in a V1 delegate action would drop the index and the
/// runtime would reject the result (`DelegateActionRequiresNonGasKey`); the fix
/// (a NEP-611 `DelegateActionV2`) needs a Ledger-app signing instruction that
/// does not exist. Reject the combination up front with a clear error. Called
/// from the two Ledger manual delegate-action branches; the software signers use
/// [`get_signed_delegate_action`], which builds a V2 delegate action instead.
pub fn ensure_ledger_gas_key_not_delegated(
    unsigned_transaction: &near_primitives::transaction::Transaction,
) -> color_eyre::eyre::Result<()> {
    if unsigned_transaction.nonce().nonce_index().is_some() {
        color_eyre::eyre::bail!(
            "Signing a meta-transaction (--sign-as-delegate-action) with a gas key is not supported on Ledger yet: the Ledger app can only sign NEP-366 delegate actions, which cannot carry a gas key's nonce index (that requires a NEP-611 DelegateV2 action). Sign with a software key, or re-run without --sign-as-delegate-action."
        );
    }
    Ok(())
}

/// Build and sign a delegate action for a meta-transaction.
///
/// Branches on the nonce: an ordinary access key (plain nonce) produces a NEP-366
/// V1 `SignedDelegateAction`; a gas key (`nonce_index.is_some()`) produces a
/// NEP-611 `VersionedSignedDelegateAction` (V2), whose `TransactionNonce` carries
/// the parallel-nonce index. Each version is signed under its own NEP-461 message
/// domain, so the two signatures are never interchangeable.
pub fn get_signed_delegate_action(
    unsigned_transaction: near_primitives::transaction::Transaction,
    public_key: &near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
    max_block_height: u64,
) -> color_eyre::eyre::Result<SignedTransactionOrSignedDelegateAction> {
    use near_primitives::signable_message::{SignableMessage, SignableMessageType};

    let sender_id = unsigned_transaction.signer_id().clone();
    let receiver_id = unsigned_transaction.receiver_id().clone();
    let delegate_public_key = unsigned_transaction.public_key().clone();
    let nonce = unsigned_transaction.nonce();

    let actions = unsigned_transaction
        .take_actions()
        .into_iter()
        .map(near_primitives::action::delegate::NonDelegateAction::try_from)
        .collect::<Result<Vec<_>, _>>()
        .expect("Internal error: can not convert the action to non delegate action (delegate action can not be delegated again).");

    let signer = near_crypto::InMemorySigner::from_secret_key(sender_id.clone(), private_key);

    let (signature, signed) = match nonce.nonce_index() {
        None => {
            // Ordinary access key: NEP-366 V1 delegate action (plain nonce).
            let delegate_action = near_primitives::action::delegate::DelegateAction {
                sender_id,
                receiver_id,
                actions,
                nonce: nonce.nonce(),
                max_block_height,
                public_key: delegate_public_key,
            };
            let signature =
                SignableMessage::new(&delegate_action, SignableMessageType::DelegateAction)
                    .sign(&signer);
            (
                signature.clone(),
                SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                    near_primitives::action::delegate::SignedDelegateAction {
                        delegate_action,
                        signature,
                    },
                ),
            )
        }
        Some(_) => {
            // Gas key: NEP-611 V2 delegate action; its nonce carries the parallel-nonce index.
            let delegate_action = near_primitives::action::delegate::DelegateActionV2 {
                sender_id,
                receiver_id,
                actions,
                nonce,
                max_block_height,
                public_key: delegate_public_key,
            };
            let payload = near_primitives::action::delegate::VersionedDelegateActionPayload::V2(
                delegate_action,
            );
            let signature =
                SignableMessage::new(&payload, SignableMessageType::DelegateActionV2).sign(&signer);
            (
                signature.clone(),
                SignedTransactionOrSignedDelegateAction::SignedDelegateActionV2(
                    near_primitives::action::delegate::VersionedSignedDelegateAction {
                        delegate_action: payload,
                        signature,
                    },
                ),
            )
        }
    };

    tracing::info!(
        parent: &tracing::Span::none(),
        "Your delegating action was signed successfully.{}",
        crate::common::indent_payload(&format!(
            "\nNote that the signed transaction is valid until block {max_block_height}. You can change the validity of a transaction by setting a flag in the command: --meta-transaction-valid-for 2000\nPublic key: {public_key}\nSignature:  {signature}\n"
        ))
    );

    Ok(signed)
}

/// The nonce (and, for gas keys, the nonce index) to use for the next transaction.
///
/// Determined once by [`resolve_online_nonce`] / [`resolve_offline_nonce`] and then
/// consumed by [`build_unsigned_transaction`] to pick the transaction version.
#[derive(Debug, Clone, Copy)]
pub enum NonceResolution {
    /// Ordinary access key: a plain nonce, builds a (V0) transaction.
    Plain {
        nonce: near_primitives::types::Nonce,
    },
    /// Gas key: a nonce at a specific parallel-nonce index, builds a V1 transaction.
    GasKey {
        nonce: near_primitives::types::Nonce,
        nonce_index: near_primitives::types::NonceIndex,
    },
}

impl NonceResolution {
    /// The nonce value, used to populate the intermediate `TransactionV0.nonce` field
    /// (which is preserved as-is for the plain path and superseded for the gas-key path).
    pub fn nonce(&self) -> near_primitives::types::Nonce {
        match self {
            NonceResolution::Plain { nonce } | NonceResolution::GasKey { nonce, .. } => *nonce,
        }
    }
}

/// `interactive-clap` only implements `ToCli` for `u64`/`u128`, not `u16`, so the
/// `--nonce-index` CLI field is collected as `u64` and narrowed here, bounded by the
/// protocol limit `AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY`.
pub fn nonce_index_from_cli(
    nonce_index: u64,
) -> color_eyre::eyre::Result<near_primitives::types::NonceIndex> {
    let max = near_primitives::account::AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY;
    if nonce_index >= u64::from(max) {
        color_eyre::eyre::bail!(
            "--nonce-index must be less than {max}, the maximum number of parallel nonces a gas key can have, got {nonce_index}"
        );
    }
    Ok(nonce_index as near_primitives::types::NonceIndex)
}

/// Returns `true` if the access key permission is a gas key.
pub(crate) fn is_gas_key_permission(
    permission: &near_primitives::views::AccessKeyPermissionView,
) -> bool {
    matches!(
        permission,
        near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess { .. }
            | near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall { .. }
    )
}

/// Online path: query the access key for `signer_id`/`public_key` and resolve the next
/// nonce, the recent block hash and block height.
///
/// For an ordinary access key this is `access_key.nonce + 1` (unchanged behavior). For a
/// gas key it queries `view_gas_key_nonces` and uses `nonces[nonce_index] + 1`; when no
/// `--nonce-index` was given, index 0 is used.
pub fn resolve_online_nonce(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    signer_id: &near_primitives::types::AccountId,
    public_key: &near_crypto::PublicKey,
    nonce_index: Option<near_primitives::types::NonceIndex>,
    network_name: &str,
) -> color_eyre::eyre::Result<(
    NonceResolution,
    near_primitives::hash::CryptoHash,
    near_primitives::types::BlockHeight,
)> {
    use crate::common::JsonRpcClientExt;
    use crate::common::RpcQueryResponseExt;
    use color_eyre::eyre::WrapErr;

    let rpc_query_response = json_rpc_client
        .blocking_call_view_access_key(
            signer_id,
            public_key,
            near_primitives::types::BlockReference::latest(),
        )
        .wrap_err_with(|| {
            format!("Cannot sign a transaction due to an error while fetching the most recent nonce value on network <{network_name}>")
        })?;
    let access_key = rpc_query_response
        .access_key_view()
        .wrap_err("Error current_nonce")?;
    let block_hash = rpc_query_response.block_hash;
    let block_height = rpc_query_response.block_height;

    let resolution = if is_gas_key_permission(&access_key.permission) {
        let nonce_index = nonce_index.unwrap_or(0);
        let gas_key_nonces = json_rpc_client
            .blocking_call_view_gas_key_nonces(
                signer_id,
                public_key,
                near_primitives::types::BlockReference::latest(),
            )
            .wrap_err_with(|| {
                format!("Cannot sign a transaction due to an error while fetching gas key nonces on network <{network_name}>")
            })?
            .gas_key_nonces_view()?;
        let current = *gas_key_nonces
            .nonces
            .get(usize::from(nonce_index))
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "Gas key has {} parallel nonce(s); --nonce-index {} is out of range",
                    gas_key_nonces.nonces.len(),
                    nonce_index
                )
            })?;
        NonceResolution::GasKey {
            nonce: current + 1,
            nonce_index,
        }
    } else {
        if let Some(nonce_index) = nonce_index {
            // Refuse rather than silently sign with a plain nonce: the user asked
            // for a specific gas-key parallel nonce, but this key isn't a gas key.
            color_eyre::eyre::bail!(
                "--nonce-index {nonce_index} was provided, but access key {public_key} on {signer_id} is not a gas key. Only gas keys have parallel nonces; omit --nonce-index to sign with an ordinary access key."
            );
        }
        NonceResolution::Plain {
            nonce: access_key.nonce + 1,
        }
    };

    Ok((resolution, block_hash, block_height))
}

/// Offline path: build the nonce resolution from a user-provided nonce and optional
/// `--nonce-index` (a gas-key nonce when an index is given, otherwise a plain nonce).
pub fn resolve_offline_nonce(
    nonce: near_primitives::types::Nonce,
    nonce_index: Option<near_primitives::types::NonceIndex>,
) -> NonceResolution {
    match nonce_index {
        Some(nonce_index) => NonceResolution::GasKey { nonce, nonce_index },
        None => NonceResolution::Plain { nonce },
    }
}

/// Wrap a (post-`on_before_signing_callback`) [`TransactionV0`] into the right transaction
/// version: V0 for an ordinary nonce, V1 with `TransactionNonce::GasKeyNonce` for a gas key.
///
/// All gas-key / versioning logic lives here so the signer modules stay version-agnostic.
pub fn build_unsigned_transaction(
    tx: near_primitives::transaction::TransactionV0,
    resolution: NonceResolution,
) -> near_primitives::transaction::Transaction {
    match resolution {
        NonceResolution::Plain { .. } => near_primitives::transaction::Transaction::V0(tx),
        NonceResolution::GasKey { nonce, nonce_index } => {
            near_primitives::transaction::Transaction::V1(
                near_primitives::transaction::TransactionV1 {
                    signer_id: tx.signer_id,
                    public_key: tx.public_key,
                    nonce: near_primitives::transaction::TransactionNonce::GasKeyNonce {
                        nonce,
                        nonce_index,
                    },
                    receiver_id: tx.receiver_id,
                    block_hash: tx.block_hash,
                    actions: tx.actions,
                    nonce_mode: near_primitives::transaction::NonceMode::default(),
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tx_v0() -> near_primitives::transaction::TransactionV0 {
        near_primitives::transaction::TransactionV0 {
            signer_id: "alice.near".parse().unwrap(),
            public_key: near_crypto::SecretKey::from_random(near_crypto::KeyType::ED25519)
                .public_key(),
            nonce: 1,
            receiver_id: "bob.near".parse().unwrap(),
            block_hash: near_primitives::hash::CryptoHash::default(),
            actions: vec![],
        }
    }

    #[test]
    fn nonce_index_from_cli_bounds() {
        let max = u64::from(near_primitives::account::AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY);
        // The largest valid index is `max - 1`; `max` and above are rejected.
        assert!(nonce_index_from_cli(max - 1).is_ok());
        assert!(nonce_index_from_cli(max).is_err());
    }

    #[test]
    fn plain_transactions_may_be_delegated_on_ledger() {
        let tx = build_unsigned_transaction(sample_tx_v0(), NonceResolution::Plain { nonce: 1 });
        assert!(ensure_ledger_gas_key_not_delegated(&tx).is_ok());
    }

    #[test]
    fn gas_key_transactions_cannot_be_delegated_on_ledger() {
        // The Ledger app can only sign NEP-366 (V1) delegate actions, which
        // can't encode a gas key's nonce_index, so the combination is refused.
        let tx = build_unsigned_transaction(
            sample_tx_v0(),
            NonceResolution::GasKey {
                nonce: 1,
                nonce_index: 0,
            },
        );
        assert!(ensure_ledger_gas_key_not_delegated(&tx).is_err());
    }

    fn sample_tx_v0_with_key(
        public_key: near_crypto::PublicKey,
        nonce: near_primitives::types::Nonce,
    ) -> near_primitives::transaction::TransactionV0 {
        near_primitives::transaction::TransactionV0 {
            public_key,
            nonce,
            ..sample_tx_v0()
        }
    }

    #[test]
    fn plain_meta_transaction_signs_delegate_v1() {
        // An ordinary access key signs a NEP-366 (V1) delegate action.
        let secret_key = near_crypto::SecretKey::from_random(near_crypto::KeyType::ED25519);
        let public_key = secret_key.public_key();
        let tx = build_unsigned_transaction(
            sample_tx_v0_with_key(public_key.clone(), 7),
            NonceResolution::Plain { nonce: 7 },
        );

        match get_signed_delegate_action(tx, &public_key, secret_key, 1000).unwrap() {
            SignedTransactionOrSignedDelegateAction::SignedDelegateAction(signed) => {
                assert!(signed.verify(), "the V1 delegate signature must verify");
                assert_eq!(signed.delegate_action.nonce, 7);
                assert_eq!(signed.delegate_action.max_block_height, 1000);
                assert_eq!(signed.delegate_action.public_key, public_key);
            }
            other => panic!("expected a V1 SignedDelegateAction, got {other:?}"),
        }
    }

    #[test]
    fn gas_key_meta_transaction_signs_delegate_v2() {
        // A gas key signs a NEP-611 (V2) delegate action whose nonce carries the
        // parallel-nonce index; the signature verifies under the V2 domain.
        let secret_key = near_crypto::SecretKey::from_random(near_crypto::KeyType::ED25519);
        let public_key = secret_key.public_key();
        let tx = build_unsigned_transaction(
            sample_tx_v0_with_key(public_key.clone(), 7),
            NonceResolution::GasKey {
                nonce: 7,
                nonce_index: 3,
            },
        );

        match get_signed_delegate_action(tx, &public_key, secret_key, 1000).unwrap() {
            SignedTransactionOrSignedDelegateAction::SignedDelegateActionV2(signed) => {
                assert!(signed.verify(), "the V2 delegate signature must verify");
                match &signed.delegate_action {
                    near_primitives::action::delegate::VersionedDelegateActionPayload::V2(
                        delegate_action,
                    ) => {
                        assert_eq!(delegate_action.nonce.nonce(), 7);
                        assert_eq!(delegate_action.nonce.nonce_index(), Some(3));
                        assert_eq!(delegate_action.max_block_height, 1000);
                        assert_eq!(delegate_action.public_key, public_key);
                    }
                }
            }
            other => panic!("expected a V2 SignedDelegateActionV2, got {other:?}"),
        }
    }
}
