use color_eyre::eyre::Context;

#[derive(Debug, Clone, derive_more::FromStr)]
pub struct FileBytes {
    inner: std::path::PathBuf,
}

impl interactive_clap::ToCli for FileBytes {
    type CliVariant = FileBytes;
}

impl std::fmt::Display for FileBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

impl FileBytes {
    pub fn try_into_bytes(&self) -> color_eyre::Result<Vec<u8>> {
        Ok(std::fs::read_to_string(self.inner.clone())
            .wrap_err_with(|| format!("Error reading data from file: {}", self.inner.display()))?
            .into_bytes())
    }
}
