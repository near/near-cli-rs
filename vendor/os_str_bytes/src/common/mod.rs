use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::result;

#[cfg(any(target_os = "hermit", target_os = "redox", unix))]
use std::os::unix as os;
#[cfg(any(target_env = "wasi", target_os = "wasi"))]
use std::os::wasi as os;

use os::ffi::OsStrExt;
use os::ffi::OsStringExt;

if_raw! {
    pub(super) mod raw;
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum EncodingError {}

impl Display for EncodingError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl Error for EncodingError {}

type Result<T> = result::Result<T, EncodingError>;

pub(crate) fn os_str_from_bytes(string: &[u8]) -> Result<Cow<'_, OsStr>> {
    Ok(Cow::Borrowed(OsStrExt::from_bytes(string)))
}

pub(crate) fn os_str_to_bytes(os_string: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(OsStrExt::as_bytes(os_string))
}

pub(crate) fn os_string_from_bytes(string: &[u8]) -> Result<OsString> {
    os_str_from_bytes(&string).map(Cow::into_owned)
}

pub(crate) fn os_string_from_vec(string: Vec<u8>) -> Result<OsString> {
    Ok(OsStringExt::from_vec(string))
}

pub(crate) fn os_string_into_vec(os_string: OsString) -> Vec<u8> {
    OsStringExt::into_vec(os_string)
}
