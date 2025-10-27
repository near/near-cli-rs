use hex::FromHex;
use near_crypto::Secp256K1Signature;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AffinePoint {
    pub affine_point: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Scalar {
    pub scalar: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SignResultSecp256K1 {
    pub big_r: AffinePoint,
    pub s: Scalar,
    pub recovery_id: u8,
}

impl From<SignResultSecp256K1> for Secp256K1Signature {
    fn from(value: SignResultSecp256K1) -> Self {
        // Get r and s from the sign result
        let big_r = value.big_r.affine_point;
        let s = value.s.scalar;

        // Remove first two bytes
        let r = &big_r[2..];

        // Convert hex to bytes
        let r_bytes = <[u8; 32]>::from_hex(r).expect("Invalid hex in r");
        let s_bytes = <[u8; 32]>::from_hex(s).expect("Invalid hex in s");

        // Add individual bytes together in the correct order
        let mut signature_bytes = [0u8; 65];
        signature_bytes[..32].copy_from_slice(&r_bytes);
        signature_bytes[32..64].copy_from_slice(&s_bytes);
        signature_bytes[64] = value.recovery_id;

        Secp256K1Signature::from(signature_bytes)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SignResultEd25519 {
    #[allow(unused)]
    pub scheme: String,
    pub signature: Vec<u8>,
}

impl From<SignResultEd25519> for ed25519_dalek::Signature {
    fn from(value: SignResultEd25519) -> Self {
        let signature_bytes: [u8; ed25519_dalek::SIGNATURE_LENGTH] = value
            .signature
            .try_into()
            .expect("Invalid signature length for Ed25519");

        // Sanity check form near_crypto
        assert!(
            signature_bytes[ed25519_dalek::SIGNATURE_LENGTH - 1] & 0b1110_0000 == 0,
            "Signature error: Sanity check failed"
        );

        ed25519_dalek::Signature::from_bytes(&signature_bytes)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SignResult {
    Secp256K1(SignResultSecp256K1),
    Ed25519(SignResultEd25519),
}

impl From<SignResult> for near_crypto::Signature {
    fn from(value: SignResult) -> Self {
        match value {
            SignResult::Secp256K1(secp) => near_crypto::Signature::SECP256K1(secp.into()),
            SignResult::Ed25519(ed) => near_crypto::Signature::ED25519(ed.into()),
        }
    }
}
