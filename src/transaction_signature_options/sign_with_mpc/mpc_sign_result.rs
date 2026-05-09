use hex::FromHex;

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

impl SignResultSecp256K1 {
    fn to_signature_bytes(&self) -> [u8; 65] {
        // Get r and s from the sign result
        let big_r = &self.big_r.affine_point;
        let s = &self.s.scalar;

        // Remove first two bytes
        let r = &big_r[2..];

        // Convert hex to bytes
        let r_bytes = <[u8; 32]>::from_hex(r).expect("Invalid hex in r");
        let s_bytes = <[u8; 32]>::from_hex(s).expect("Invalid hex in s");

        // Add individual bytes together in the correct order
        let mut signature_bytes = [0u8; 65];
        signature_bytes[..32].copy_from_slice(&r_bytes);
        signature_bytes[32..64].copy_from_slice(&s_bytes);
        signature_bytes[64] = self.recovery_id;

        signature_bytes
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SignResultEd25519 {
    #[allow(unused)]
    pub scheme: String,
    pub signature: Vec<u8>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SignResult {
    Secp256K1(SignResultSecp256K1),
    Ed25519(SignResultEd25519),
}

impl From<SignResult> for near_kit::Signature {
    fn from(value: SignResult) -> Self {
        match value {
            SignResult::Secp256K1(secp) => {
                near_kit::Signature::secp256k1_from_bytes(secp.to_signature_bytes())
            }
            SignResult::Ed25519(ed) => {
                let signature_bytes: [u8; ed25519_dalek::SIGNATURE_LENGTH] = ed
                    .signature
                    .try_into()
                    .expect("Invalid signature length for Ed25519");
                // Sanity check from near_crypto
                assert!(
                    signature_bytes[ed25519_dalek::SIGNATURE_LENGTH - 1] & 0b1110_0000 == 0,
                    "Signature error: Sanity check failed"
                );
                near_kit::Signature::ed25519_from_bytes(signature_bytes)
            }
        }
    }
}
