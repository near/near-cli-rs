use std::convert::TryInto;

#[derive(
    Debug,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    smart_default::SmartDefault,
)]
#[strum(serialize_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Plaintext,
    Json,
}

#[derive(
    Debug,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    smart_default::SmartDefault,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionFormat {
    #[default]
    Base64,
    Hex,
}

#[derive(derive_more::AsRef)]
pub struct BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    inner: T,
}

impl<T> std::fmt::Debug for BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        near_primitives::serialize::to_base(self.inner.as_ref()).fmt(f)
    }
}

impl<T> std::str::FromStr for BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base(value)
                .map_err(|_| "err")?
                .as_slice()
                .try_into()
                .map_err(|_| "err")?,
        })
    }
}

impl<T> BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    pub fn into_inner(self) -> T {
        self.inner
    }
}
