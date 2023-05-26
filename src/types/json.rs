use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Json(pub Value);

impl From<Json> for Value {
    fn from(item: Json) -> Self {
        item.0
    }
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Json {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let json: Value = serde_json::from_str(s).map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(json))
    }
}

impl interactive_clap::ToCli for Json {
    type CliVariant = Json;
}
