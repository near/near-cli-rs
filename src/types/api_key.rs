#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub struct ApiKey(pub near_jsonrpc_client::auth::ApiKey);

impl From<ApiKey> for near_jsonrpc_client::auth::ApiKey {
    fn from(api_key: ApiKey) -> Self {
        api_key.0
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Never fail: `to_string()` panics if `fmt` returns an error.
        f.write_str(&String::from_utf8_lossy(self.0.as_bytes()))
    }
}

impl std::str::FromStr for ApiKey {
    type Err = color_eyre::eyre::Report;

    fn from_str(api_key: &str) -> Result<Self, Self::Err> {
        let api_key = near_jsonrpc_client::auth::ApiKey::new(api_key)?;
        // A header value may contain non-ASCII bytes, but saving the key to
        // config.toml (`Serialize`) needs `to_str()`, which only accepts
        // visible ASCII.
        api_key.to_str().map_err(|_| {
            color_eyre::eyre::eyre!("The API key must contain only visible ASCII characters")
        })?;
        Ok(Self(api_key))
    }
}

impl serde::ser::Serialize for ApiKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.0.to_str().map_err(serde::ser::Error::custom)?)
    }
}

impl<'de> serde::de::Deserialize<'de> for ApiKey {
    fn deserialize<D>(deserializer: D) -> Result<ApiKey, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // Stay lenient for keys already saved in a config file; only new input
        // is restricted to visible ASCII (see `FromStr`).
        let api_key = String::deserialize(deserializer)?;
        near_jsonrpc_client::auth::ApiKey::new(api_key)
            .map(Self)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl interactive_clap::ToCli for ApiKey {
    type CliVariant = ApiKey;
}

#[cfg(test)]
mod tests {
    use super::ApiKey;
    use std::str::FromStr;

    #[test]
    fn non_ascii_api_key_is_rejected() {
        for key in ["é", "a😕b", "ключ"] {
            assert!(ApiKey::from_str(key).is_err(), "{key}");
        }
    }

    #[test]
    fn saved_non_ascii_api_key_still_loads_and_displays() {
        let key: ApiKey = serde_json::from_str("\"é\"").unwrap();
        assert_eq!(key.to_string(), "é");
    }

    #[test]
    fn ascii_api_key_round_trips() {
        let key = ApiKey::from_str("abc-123_XYZ").unwrap();
        assert_eq!(key.to_string(), "abc-123_XYZ");
    }
}
