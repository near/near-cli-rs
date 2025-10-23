use hex::FromHex;
use near_crypto::Secp256K1Signature;

impl SignRequest {
    pub fn new(payload: [u8; 32], path: String, key_version: u32) -> Self {
        Self {
            payload,
            path,
            key_version,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AffinePoint {
    pub affine_point: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Scalar {
    pub scalar: String,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SignResult {
    pub big_r: AffinePoint,
    pub s: Scalar,
    pub recovery_id: u8,
}

impl From<SignResult> for Secp256K1Signature {
    fn from(value: SignResult) -> Self {
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
