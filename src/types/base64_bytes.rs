#[derive(Debug, Clone)]
pub struct Base64Bytes {
    pub inner: Vec<u8>,
}

impl interactive_clap::ToCli for Base64Bytes {
    type CliVariant = Base64Bytes;
}

impl std::str::FromStr for Base64Bytes {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base64(s).map_err(|err| {
                format!(
                    "parsing action {s} failed due to invalid base64 sequence: {}",
                    err
                )
            })?,
        })
    }
}

impl std::fmt::Display for Base64Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", near_primitives::serialize::to_base64(&self.inner))
    }
}
