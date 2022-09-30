#[derive(Debug, Clone)]
pub struct Url(pub url::Url);

impl From<Url> for url::Url {
    fn from(url: Url) -> Self {
        url.0
    }
}

impl From<url::Url> for Url {
    fn from(url: url::Url) -> Self {
        Self(url)
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Url {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = url::Url::parse(s)?;
        Ok(Self(url))
    }
}

impl interactive_clap::ToCli for Url {
    type CliVariant = Url;
}
