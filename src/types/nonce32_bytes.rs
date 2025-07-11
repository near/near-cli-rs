#[derive(Debug, Clone, Default)]
pub struct Nonce32 {
    inner: [u8; 32],
}

impl std::str::FromStr for Nonce32 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = near_primitives::serialize::from_base64(s)
            .map_err(|err| format!("Invalid base64: {err}"))?;

        if bytes.len() != 32 {
            return Err(format!(
                "Invalid nonce length: expected 32 bytes, got {}",
                bytes.len()
            ));
        }

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&bytes);
        Ok(Self { inner: nonce })
    }
}

impl std::fmt::Display for Nonce32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", near_primitives::serialize::to_base64(&self.inner))
    }
}

impl Nonce32 {
    pub fn as_array(&self) -> [u8; 32] {
        self.inner
    }
}

impl interactive_clap::ToCli for Nonce32 {
    type CliVariant = Nonce32;
}
