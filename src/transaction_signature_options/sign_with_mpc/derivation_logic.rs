use sha3::{Digest, Sha3_256};

const TWEAK_DERIVATION_PREFIX: &str = "near-mpc-recovery v0.1.0 epsilon derivation:";

// Create a global secp256k1 context (same as near_crypto does internally)
static SECP256K1: std::sync::LazyLock<secp256k1::Secp256k1<secp256k1::All>> =
    std::sync::LazyLock::new(secp256k1::Secp256k1::new);

/// Derives a tweak from the predecessor account ID and derivation path
fn derive_tweak(predecessor_id: &near_primitives::types::AccountId, path: &str) -> [u8; 32] {
    // this is stolen from mpc contract code, and predecessor/path must be already verified Near AccountId
    // https://github.com/near/mpc/blob/main/crates/contract/src/crypto_shared/kdf.rs#L16
    let derivation_path = format!("{TWEAK_DERIVATION_PREFIX}{predecessor_id},{path}");
    let mut hasher = Sha3_256::new();
    hasher.update(derivation_path.as_bytes());
    hasher.finalize().into()
}

#[tracing::instrument(name = "Deriving Secp256K1 PublicKey ...", skip_all)]
pub fn derive_public_key(
    mpc_contract_public_key: &near_crypto::Secp256K1PublicKey,
    admin_account_id: &near_primitives::types::AccountId,
    path: &str,
) -> color_eyre::eyre::Result<near_crypto::Secp256K1PublicKey> {
    // deriving tweak from the predecessor and path
    let tweak_bytes = derive_tweak(admin_account_id, path);

    // create SecretKey that is scalar for our func
    // supposed to behave like k256::Scalar::from_repr
    // https://github.com/near/mpc/blob/main/crates/contract/src/crypto_shared/kdf.rs#L39
    let tweak_scalar = secp256k1::SecretKey::from_slice(&tweak_bytes)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid tweak scalar: {}", e))?;

    // converting near_crypto key to secp256k1 pub key
    // + adding 0x04 at beginning to show that it's uncompressed
    // near_public_key_to_affine_point
    // https://github.com/near/mpc/blob/main/crates/contract/src/crypto_shared.rs#L24
    let mut uncompressed_bytes = [0u8; 65];
    uncompressed_bytes[0] = 0x04;
    uncompressed_bytes[1..65].copy_from_slice(mpc_contract_public_key.as_ref());

    let mpc_pk_secp = secp256k1::PublicKey::from_slice(&uncompressed_bytes)
        .map_err(|err| color_eyre::eyre::eyre!("Invalid parent public key: {}", err))?;

    // tweak * G, where G is generator point from Secp
    // <Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * tweak
    // https://github.com/near/mpc/blob/main/crates/contract/src/crypto_shared/kdf.rs#L44C11-L44C75
    let tweak_pk = secp256k1::PublicKey::from_secret_key(&SECP256K1, &tweak_scalar);

    // adding two pks: derived = (tweak * G) + parent
    // https://github.com/near/mpc/blob/main/crates/contract/src/crypto_shared/kdf.rs#L44C10-L44C89
    let derived_key = tweak_pk
        .combine(&mpc_pk_secp)
        .map_err(|err| color_eyre::eyre::eyre!("Failed to combine public keys: {}", err))?;

    // converting back to near_crypto format without 0x04
    let derived_key_serialized = derived_key.serialize_uncompressed();

    near_crypto::Secp256K1PublicKey::try_from(&derived_key_serialized[1..65])
        .map_err(|err| color_eyre::eyre::eyre!("Failed to create derived key: {}", err))
}
