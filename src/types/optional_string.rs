#[derive(Debug, Default, Clone)]
pub struct OptionalString(Option<String>);

impl std::fmt::Display for OptionalString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(s) = &self.0 {
            s.fmt(f)
        } else {
            write!(f, "")
        }
    }
}

impl std::str::FromStr for OptionalString {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(trimmed.to_lowercase())))
        }
    }
}

impl From<OptionalString> for Option<String> {
    fn from(item: OptionalString) -> Self {
        item.0
    }
}

impl From<Option<String>> for OptionalString {
    fn from(item: Option<String>) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for OptionalString {
    type CliVariant = OptionalString;
}
