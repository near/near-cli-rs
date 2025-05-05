use std::{ops::Deref, path::PathBuf};

use color_eyre::eyre::Context;

#[derive(
    Debug,
    Default,
    Clone,
    derive_more::AsRef,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
)]
#[as_ref(forward)]
pub struct Utf8PathBuf(camino::Utf8PathBuf);

impl std::fmt::Display for Utf8PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl interactive_clap::ToCli for Utf8PathBuf {
    type CliVariant = Utf8PathBuf;
}

impl Deref for Utf8PathBuf {
    type Target = camino::Utf8PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Utf8PathBuf {
    pub fn read_bytes(&self) -> color_eyre::Result<Vec<u8>> {
        std::fs::read(self.0.clone().into_std_path_buf())
            .wrap_err_with(|| format!("Error reading data from file: {:?}", self.0))
    }

    pub fn from_path_buf(path: PathBuf) -> Result<Self, PathBuf> {
        Ok(camino::Utf8PathBuf::from_path_buf(path)?.into())
    }

    pub fn join(&self, path: impl AsRef<camino::Utf8Path>) -> Utf8PathBuf {
        camino::Utf8Path::join(self.0.as_path(), path).into()
    }
}
