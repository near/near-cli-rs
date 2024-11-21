use color_eyre::eyre::WrapErr;
use serde_json::Value;

#[derive(Debug, Clone, derive_more::FromStr)]
pub struct Json {
    inner: Value,
}

impl From<Json> for Value {
    fn from(item: Json) -> Self {
        item.inner
    }
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl interactive_clap::ToCli for Json {
    type CliVariant = Json;
}

impl Json {
    pub fn try_into_bytes(&self) -> color_eyre::Result<Vec<u8>> {
        serde_json::to_vec(&self.inner).wrap_err("Data not in JSON format!")
    }
}
