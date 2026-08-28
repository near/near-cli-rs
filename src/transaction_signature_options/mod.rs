use base64::Engine as _;
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
    pub public_key: near_kit::PublicKey,
    pub private_key: near_kit::SecretKey,
}

pub type OnBeforeSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &SignedTransactionOrSignedDelegateAction,
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<String>,
>;

pub type OnAfterSendingTransactionCallback = std::sync::Arc<
    dyn Fn(&near_kit::FinalExecutionOutcome, &crate::config::NetworkConfig) -> crate::CliResult,
>;

pub type OnSendingDelegateActionCallback = std::sync::Arc<
    dyn Fn(near_kit::SignedDelegateAction, &crate::config::NetworkConfig) -> crate::CliResult,
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

/// A signed transaction (including gas-key/V1 transactions) or a V1 delegate action.
#[derive(Debug, Clone)]
pub enum SignedTransactionOrSignedDelegateAction {
    SignedTransaction(near_kit::SignedTransactionV1),
    SignedDelegateAction(near_kit::SignedDelegateAction),
}

impl serde::Serialize for SignedTransactionOrSignedDelegateAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::SignedTransaction(transaction) => {
                map.serialize_entry("SignedTransaction", &transaction.to_base64())?;
            }
            Self::SignedDelegateAction(delegate_action) => {
                let base64 = base64::engine::general_purpose::STANDARD.encode(
                    borsh::to_vec(delegate_action)
                        .expect("signed delegate action serialization should not fail"),
                );
                map.serialize_entry("SignedDelegateAction", &base64)?;
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for SignedTransactionOrSignedDelegateAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: std::collections::HashMap<String, String> =
            serde::Deserialize::deserialize(deserializer)?;

        if let Some(base64) = map.get("SignedTransaction") {
            let transaction = near_kit::SignedTransactionV1::from_base64(base64)
                .map_err(serde::de::Error::custom)?;
            Ok(Self::SignedTransaction(transaction))
        } else if let Some(base64) = map.get("SignedDelegateAction") {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(base64)
                .map_err(serde::de::Error::custom)?;
            let delegate_action = borsh::from_slice(&bytes).map_err(serde::de::Error::custom)?;
            Ok(Self::SignedDelegateAction(delegate_action))
        } else {
            Err(serde::de::Error::custom(
                "expected SignedTransaction or SignedDelegateAction key",
            ))
        }
    }
}

impl From<near_kit::SignedTransactionV1> for SignedTransactionOrSignedDelegateAction {
    fn from(signed_transaction: near_kit::SignedTransactionV1) -> Self {
        Self::SignedTransaction(signed_transaction)
    }
}

impl From<near_kit::SignedDelegateAction> for SignedTransactionOrSignedDelegateAction {
    fn from(signed_delegate_action: near_kit::SignedDelegateAction) -> Self {
        Self::SignedDelegateAction(signed_delegate_action)
    }
}

/// Reject gas-key meta-transactions until the CLI supports DelegateV2 signing.
///
/// A V1 `DelegateAction` only carries a plain nonce. Converting a gas-key
/// transaction to it would discard the nonce index and the runtime would reject
/// it with `DelegateActionRequiresNonGasKey`.
pub fn ensure_gas_key_not_delegated(
    unsigned_transaction: &near_kit::VersionedTransaction,
) -> color_eyre::eyre::Result<()> {
    if unsigned_transaction.nonce().nonce_index().is_some() {
        color_eyre::eyre::bail!(
            "Signing with a gas key is not supported for meta-transactions (--sign-as-delegate-action) yet: a DelegateAction cannot carry the gas key's nonce index (requires DelegateV2). Re-run without --sign-as-delegate-action, or sign with an ordinary access key."
        );
    }
    Ok(())
}

pub fn get_signed_delegate_action(
    unsigned_transaction: near_kit::VersionedTransaction,
    public_key: &near_kit::PublicKey,
    private_key: near_kit::SecretKey,
    max_block_height: u64,
) -> color_eyre::eyre::Result<near_kit::SignedDelegateAction> {
    ensure_gas_key_not_delegated(&unsigned_transaction)?;

    let delegate_action = near_kit::DelegateAction {
        sender_id: unsigned_transaction.signer_id().clone(),
        receiver_id: unsigned_transaction.receiver_id().clone(),
        actions: unsigned_transaction
            .actions()
            .iter()
            .cloned()
            .map(near_kit::NonDelegateAction::try_from)
            .collect::<Result<_, _>>()
            .expect(
                "Internal error: can not convert the action to non delegate action (delegate action can not be delegated again).",
            ),
        nonce: unsigned_transaction.nonce().nonce(),
        max_block_height,
        public_key: unsigned_transaction.public_key().clone(),
    };

    let signature = private_key.sign(delegate_action.get_hash().as_bytes());

    tracing::info!(
        parent: &tracing::Span::none(),
        "Your delegating action was signed successfully.{}",
        crate::common::indent_payload(&format!(
            "\nNote that the signed transaction is valid until block {max_block_height}. You can change the validity of a transaction by setting a flag in the command: --meta-transaction-valid-for 2000\nPublic key: {public_key}\nSignature:  {signature}\n"
        ))
    );

    Ok(delegate_action.sign(signature))
}

/// The nonce (and, for gas keys, nonce index) for the next transaction.
#[derive(Debug, Clone, Copy)]
pub enum NonceResolution {
    /// Ordinary access key: build a legacy V0 transaction.
    Plain { nonce: near_kit::Nonce },
    /// Gas key: build a V1 transaction with a parallel nonce index.
    GasKey {
        nonce: near_kit::Nonce,
        nonce_index: near_kit::NonceIndex,
    },
}

impl NonceResolution {
    pub fn nonce(&self) -> near_kit::Nonce {
        match self {
            Self::Plain { nonce } | Self::GasKey { nonce, .. } => *nonce,
        }
    }
}

/// Narrow the interactive-clap `u64` nonce index to the protocol's `u16` type.
pub fn nonce_index_from_cli(nonce_index: u64) -> color_eyre::eyre::Result<near_kit::NonceIndex> {
    let max = near_kit::MAX_NONCES_FOR_GAS_KEY;
    if nonce_index >= u64::from(max) {
        color_eyre::eyre::bail!(
            "--nonce-index must be less than {max}, the maximum number of parallel nonces a gas key can have, got {nonce_index}"
        );
    }
    Ok(nonce_index as near_kit::NonceIndex)
}

/// Returns `true` if the access key permission is a gas key.
pub(crate) fn is_gas_key_permission(permission: &near_kit::AccessKeyPermissionView) -> bool {
    matches!(
        permission,
        near_kit::AccessKeyPermissionView::GasKeyFullAccess { .. }
            | near_kit::AccessKeyPermissionView::GasKeyFunctionCall { .. }
    )
}

/// Resolve the next nonce and recent block for an online signer.
///
/// Ordinary keys use `access_key.nonce + 1`. Gas keys query their parallel
/// nonce array and use `nonces[nonce_index] + 1`, defaulting to index zero.
pub fn resolve_online_nonce(
    network_config: &crate::config::NetworkConfig,
    signer_id: &near_kit::AccountId,
    public_key: &near_kit::PublicKey,
    nonce_index: Option<near_kit::NonceIndex>,
) -> color_eyre::eyre::Result<(NonceResolution, near_kit::CryptoHash, u64)> {
    use crate::common::{RpcResultExt, block_on};
    use color_eyre::eyre::WrapErr;

    let client = network_config.client();
    let rpc = client.rpc();
    let access_key = block_on(rpc.view_access_key(
        signer_id,
        public_key,
        near_kit::BlockReference::optimistic(),
    ))
    .into_eyre()
    .wrap_err_with(|| {
        format!(
            "Cannot sign a transaction due to an error while fetching the most recent nonce value on network <{}>",
            network_config.network_name
        )
    })?;

    let resolution = if is_gas_key_permission(&access_key.permission) {
        let nonce_index = nonce_index.unwrap_or(0);
        let gas_key_nonces = block_on(rpc.view_gas_key_nonces(
            signer_id,
            public_key,
            near_kit::BlockReference::optimistic(),
        ))
        .into_eyre()
        .wrap_err_with(|| {
            format!(
                "Cannot sign a transaction due to an error while fetching gas key nonces on network <{}>",
                network_config.network_name
            )
        })?;
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
            color_eyre::eyre::bail!(
                "--nonce-index {nonce_index} was provided, but access key {public_key} on {signer_id} is not a gas key. Only gas keys have parallel nonces; omit --nonce-index to sign with an ordinary access key."
            );
        }
        NonceResolution::Plain {
            nonce: access_key.nonce + 1,
        }
    };

    Ok((resolution, access_key.block_hash, access_key.block_height))
}

/// Resolve a user-provided offline nonce and optional gas-key nonce index.
pub fn resolve_offline_nonce(
    nonce: near_kit::Nonce,
    nonce_index: Option<near_kit::NonceIndex>,
) -> NonceResolution {
    match nonce_index {
        Some(nonce_index) => NonceResolution::GasKey { nonce, nonce_index },
        None => NonceResolution::Plain { nonce },
    }
}

/// Wrap a post-callback V0 transaction in the correct wire transaction version.
pub fn build_unsigned_transaction(
    mut transaction: near_kit::Transaction,
    resolution: NonceResolution,
) -> near_kit::VersionedTransaction {
    transaction.nonce = resolution.nonce();
    match resolution {
        NonceResolution::Plain { .. } => near_kit::VersionedTransaction::V0(transaction),
        NonceResolution::GasKey { nonce_index, .. } => near_kit::VersionedTransaction::V1(
            transaction.into_gas_key_v1(nonce_index, near_kit::TransactionNonceMode::default()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transaction() -> near_kit::Transaction {
        near_kit::Transaction {
            signer_id: "alice.near".parse().unwrap(),
            public_key: near_kit::SecretKey::generate_ed25519().public_key(),
            nonce: 1,
            receiver_id: "bob.near".parse().unwrap(),
            block_hash: near_kit::CryptoHash::default(),
            actions: vec![],
        }
    }

    #[test]
    fn nonce_index_from_cli_bounds() {
        let max = u64::from(near_kit::MAX_NONCES_FOR_GAS_KEY);
        assert!(nonce_index_from_cli(max - 1).is_ok());
        assert!(nonce_index_from_cli(max).is_err());
    }

    #[test]
    fn plain_transactions_may_be_delegated() {
        let transaction =
            build_unsigned_transaction(sample_transaction(), NonceResolution::Plain { nonce: 1 });
        assert!(ensure_gas_key_not_delegated(&transaction).is_ok());
    }

    #[test]
    fn gas_key_transactions_cannot_be_delegated() {
        let transaction = build_unsigned_transaction(
            sample_transaction(),
            NonceResolution::GasKey {
                nonce: 1,
                nonce_index: 0,
            },
        );
        assert!(ensure_gas_key_not_delegated(&transaction).is_err());
    }
}
