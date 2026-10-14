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

impl TryFrom<SignResultSecp256K1> for Secp256K1Signature {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: SignResultSecp256K1) -> Result<Self, Self::Error> {
        // `big_r` is a SEC1 compressed point: a one-byte prefix (02/03) followed by the
        // 32-byte x coordinate, which is the `r` component of the signature.
        let big_r = value.big_r.affine_point;
        let r = big_r.get(2..).ok_or_else(|| {
            color_eyre::eyre::eyre!("MPC secp256k1 sign result has a too short `big_r`")
        })?;

        let r_bytes = <[u8; 32]>::from_hex(r).map_err(|err| {
            color_eyre::eyre::eyre!("Invalid hex in `big_r` of MPC sign result: {err}")
        })?;
        let s_bytes = <[u8; 32]>::from_hex(&value.s.scalar).map_err(|err| {
            color_eyre::eyre::eyre!("Invalid hex in `s` of MPC sign result: {err}")
        })?;

        let mut signature_bytes = [0u8; 65];
        signature_bytes[..32].copy_from_slice(&r_bytes);
        signature_bytes[32..64].copy_from_slice(&s_bytes);
        signature_bytes[64] = value.recovery_id;

        Ok(Secp256K1Signature::from(signature_bytes))
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SignResultEd25519 {
    #[allow(unused)]
    pub scheme: String,
    pub signature: Vec<u8>,
}

impl TryFrom<SignResultEd25519> for ed25519_dalek::Signature {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: SignResultEd25519) -> Result<Self, Self::Error> {
        let signature_bytes: [u8; ed25519_dalek::SIGNATURE_LENGTH] =
            value.signature.try_into().map_err(|bytes: Vec<u8>| {
                color_eyre::eyre::eyre!(
                    "Invalid ed25519 signature length in MPC sign result: expected {} bytes, got {}",
                    ed25519_dalek::SIGNATURE_LENGTH,
                    bytes.len()
                )
            })?;

        // Sanity check from near_crypto
        if signature_bytes[ed25519_dalek::SIGNATURE_LENGTH - 1] & 0b1110_0000 != 0 {
            return Err(color_eyre::eyre::eyre!(
                "Invalid ed25519 signature in MPC sign result: sanity check failed"
            ));
        }

        Ok(ed25519_dalek::Signature::from_bytes(&signature_bytes))
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SignResult {
    Secp256K1(SignResultSecp256K1),
    Ed25519(SignResultEd25519),
}

impl TryFrom<SignResult> for near_crypto::Signature {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: SignResult) -> Result<Self, Self::Error> {
        match value {
            SignResult::Secp256K1(secp) => Ok(near_crypto::Signature::SECP256K1(secp.try_into()?)),
            SignResult::Ed25519(ed) => Ok(near_crypto::Signature::ED25519(ed.try_into()?)),
        }
    }
}
