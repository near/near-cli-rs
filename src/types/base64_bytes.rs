use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(Debug, Clone)]
pub struct Base64Bytes {
    inner: Vec<u8>,
}

impl interactive_clap::ToCli for Base64Bytes {
    type CliVariant = Base64Bytes;
}

impl std::str::FromStr for Base64Bytes {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: STANDARD.decode(s).map_err(|err| {
                format!("parsing action {s} failed due to invalid base64 sequence: {err}")
            })?,
        })
    }
}

impl std::fmt::Display for Base64Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", STANDARD.encode(&self.inner))
    }
}

impl Base64Bytes {
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }
}
