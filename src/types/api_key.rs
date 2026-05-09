#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ApiKey {
    type Err = color_eyre::eyre::Report;

    fn from_str(api_key: &str) -> Result<Self, Self::Err> {
        if api_key.is_empty() {
            return Err(color_eyre::eyre::eyre!("API key cannot be empty"));
        }
        Ok(Self(api_key.to_string()))
    }
}

impl serde::ser::Serialize for ApiKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::de::Deserialize<'de> for ApiKey {
    fn deserialize<D>(deserializer: D) -> Result<ApiKey, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|err: color_eyre::eyre::Report| serde::de::Error::custom(err.to_string()))
    }
}

impl interactive_clap::ToCli for ApiKey {
    type CliVariant = ApiKey;
}
