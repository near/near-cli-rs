use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone)]
pub struct Base64String {
    pub inner: String,
}

impl interactive_clap::ToCli for Base64String {
    type CliVariant = Base64String;
}

impl std::str::FromStr for Base64String {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: String::from_utf8(near_primitives::serialize::from_base64(s).map_err(
                |err| {
                    format!(
                        "parsing action {s} failed due to invalid base64 sequence: {}",
                        err
                    )
                },
            )?)
            .map_err(|err| format!("Base64String not be deserialized from utf8: {}", err))?,
        })
    }
}

impl std::fmt::Display for Base64String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_string = near_primitives::serialize::to_base64(
            &self
                .inner
                .try_to_vec()
                .expect("Base64String is not expected to fail on serialization"),
        );
        write!(f, "{}", base64_string)
    }
}
