//! This crate allows interacting with the data stored internally by [`OsStr`]
//! and [`OsString`], without resorting to panics or corruption for invalid
//! UTF-8. Thus, methods can be used that are already defined on
//! [`[u8]`][slice] and [`Vec<u8>`].
//!
//! Typically, the only way to losslessly construct [`OsStr`] or [`OsString`]
//! from a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which
//! requires the bytes to be valid in UTF-8. However, since this crate makes
//! conversions directly between the platform encoding and raw bytes, even some
//! strings invalid in UTF-8 can be converted.
//!
//! # Encoding
//!
//! The encoding of bytes returned or accepted by methods of this crate is
//! intentionally left unspecified. It may vary for different platforms, so
//! defining it would run contrary to the goal of generic string handling.
//! However, the following invariants will always be upheld:
//!
//! - The encoding will be compatible with UTF-8. In particular, splitting an
//!   encoded byte sequence by a UTF-8–encoded character always produces other
//!   valid byte sequences. They can be re-encoded without error using
//!   [`OsStrBytes::from_bytes`] and similar methods.
//!
//! - All characters valid in platform strings are representable. [`OsStr`] and
//!   [`OsString`] can always be losslessly reconstructed from extracted bytes.
//!
//! Note that the chosen encoding may not match how Rust stores these strings
//! internally, which is undocumented. For instance, the result of calling
//! [`OsStr::len`] will not necessarily match the number of bytes this crate
//! uses to represent the same string.
//!
//! Additionally, concatenation may yield unexpected results without a UTF-8
//! separator. If two platform strings need to be concatenated, the only safe
//! way to do so is using [`OsString::push`]. This limitation also makes it
//! undesirable to use the bytes in interchange unless absolutely necessary. If
//! the strings need to be written as output, crate [print\_bytes] can do so
//! more safely than directly writing the bytes.
//!
//! # User Input
//!
//! Traits in this crate should ideally not be used to convert byte sequences
//! that did not originate from [`OsStr`] or a related struct. The encoding
//! used by this crate is an implementation detail, so it does not make sense
//! to expose it to users.
//!
//! Crate [bstr] offers some useful alternative methods, such as
//! [`ByteSlice::to_os_str`] and [`ByteVec::into_os_string`], that are meant
//! for user input. But, they reject some byte sequences used to represent
//! valid platform strings, which would be undesirable for reliable path
//! handling. They are best used only when accepting unknown input.
//!
//! This crate is meant to help when you already have an instance of [`OsStr`]
//! and need to modify the data in a lossless way.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file.
//!
//! ### Optional Features
//!
//! - **raw** -
//!   Enables use of the [`raw`] module.
//!
//! # Implementation
//!
//! Some methods return [`Cow`] to account for platform differences. However,
//! no guarantee is made that the same variant of that enum will always be
//! returned for the same platform. Whichever can be constructed most
//! efficiently will be returned.
//!
//! All traits are [sealed], meaning that they can only be implemented by this
//! crate. Otherwise, backward compatibility would be more difficult to
//! maintain for new features.
//!
//! # Complexity
//!
//! The time complexities of methods will vary based on what functionality is
//! available for the platform. The most efficient implementation will be used,
//! but it is important to use the most applicable method. For example,
//! [`OsStringBytes::from_vec`] will be at least as efficient as
//! [`OsStringBytes::from_bytes`], but the latter should be used when only a
//! slice is available.
//!
//! # Examples
//!
//! ```
//! use std::env;
//! use std::fs;
//! # use std::io::Result;
//!
//! use os_str_bytes::OsStrBytes;
//!
//! # fn main() -> Result<()> {
//! #     mod env {
//! #         use std::ffi::OsString;
//! #
//! #         pub fn args_os() -> impl Iterator<Item = OsString> {
//! #             let mut file = super::env::temp_dir();
//! #             file.push("os_str_bytes\u{E9}.txt");
//! #             return vec![OsString::new(), file.into_os_string()]
//! #                 .into_iter();
//! #         }
//! #     }
//! #
//! for file in env::args_os().skip(1) {
//!     if file.to_bytes().first() != Some(&b'-') {
//!         let string = "Hello, world!";
//!         fs::write(&file, string)?;
//!         assert_eq!(string, fs::read_to_string(file)?);
//!     }
//! }
//! #     Ok(())
//! # }
//! ```
//!
//! [bstr]: https://crates.io/crates/bstr
//! [`ByteSlice::to_os_str`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteSlice.html#method.to_os_str
//! [`ByteVec::into_os_string`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteVec.html#method.into_os_string
//! [`Cow`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
//! [`raw`]: raw/index.html
//! [slice]: https://doc.rust-lang.org/std/primitive.slice.html
//! [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
//! [`OsStr::len`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.len
//! [`OsStrBytes::from_bytes`]: trait.OsStrBytes.html#tymethod.from_bytes
//! [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
//! [`OsString::push`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html#method.push
//! [`OsStringBytes::from_bytes`]: trait.OsStringBytes.html#tymethod.from_bytes
//! [`OsStringBytes::from_vec`]: trait.OsStringBytes.html#tymethod.from_vec
//! [print\_bytes]: https://crates.io/crates/print_bytes
//! [`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html

#![allow(clippy::map_clone)]
#![doc(html_root_url = "https://docs.rs/os_str_bytes/*")]
// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
// https://github.com/dylni/os_str_bytes/issues/2
#![cfg_attr(os_str_bytes_docs_rs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(unused_results)]

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::result;

macro_rules! if_raw {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "raw")]
            $item
        )+
    };
}

#[cfg_attr(
    all(
        target_arch = "wasm32",
        any(target_os = "emscripten", target_os = "unknown"),
    ),
    path = "wasm32/mod.rs"
)]
#[cfg_attr(windows, path = "windows/mod.rs")]
#[cfg_attr(
    not(any(
        all(
            target_arch = "wasm32",
            any(target_os = "emscripten", target_os = "unknown"),
        ),
        windows,
    )),
    path = "common/mod.rs"
)]
mod imp;

if_raw! {
    pub mod raw;
}

/// The error that occurs when a byte sequence is not representable in the
/// platform encoding.
///
/// [`Result::unwrap`] should almost always be called on results containing
/// this error. It should be known whether or not byte sequences are properly
/// encoded for the platform, since [the module-level documentation][encoding]
/// discourages using encoded bytes in interchange. Results are returned
/// primarily to make panicking behavior explicit.
///
/// On Unix, this error is never returned, but [`OsStrExt`] or [`OsStringExt`]
/// should be used instead if that needs to be guaranteed.
///
/// [encoding]: index.html#encoding
/// [`OsStrExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStrExt.html
/// [`OsStringExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStringExt.html
/// [`Result::unwrap`]: https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap
#[derive(Debug, Eq, PartialEq)]
pub struct EncodingError(imp::EncodingError);

impl Display for EncodingError {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Error for EncodingError {}

type Result<T> = result::Result<T, EncodingError>;

/// A platform agnostic variant of [`OsStrExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: index.html
/// [`OsStrExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStrExt.html
pub trait OsStrBytes: private::Sealed + ToOwned {
    /// Converts a byte slice into an equivalent platform-native string.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsStr;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.to_bytes();
    /// assert_eq!(os_string, OsStr::from_bytes(&os_bytes).unwrap());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    fn from_bytes<TString>(string: &TString) -> Result<Cow<'_, Self>>
    where
        TString: AsRef<[u8]> + ?Sized;

    /// Converts a platform-native string into an equivalent byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// println!("{:?}", os_string.to_bytes());
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    fn to_bytes(&self) -> Cow<'_, [u8]>;
}

impl OsStrBytes for OsStr {
    #[inline]
    fn from_bytes<TString>(string: &TString) -> Result<Cow<'_, Self>>
    where
        TString: AsRef<[u8]> + ?Sized,
    {
        imp::os_str_from_bytes(string.as_ref()).map_err(EncodingError)
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        imp::os_str_to_bytes(self)
    }
}

impl OsStrBytes for Path {
    #[inline]
    fn from_bytes<TString>(string: &TString) -> Result<Cow<'_, Self>>
    where
        TString: AsRef<[u8]> + ?Sized,
    {
        OsStr::from_bytes(string).map(|os_string| match os_string {
            Cow::Borrowed(os_string) => Cow::Borrowed(Self::new(os_string)),
            Cow::Owned(os_string) => Cow::Owned(os_string.into()),
        })
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.as_os_str().to_bytes()
    }
}

/// A platform agnostic variant of [`OsStringExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: index.html
/// [`OsStringExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStringExt.html
pub trait OsStringBytes: private::Sealed + Sized {
    /// Copies a byte slice into an equivalent platform-native string.
    ///
    /// It is always better to use [`from_cow`] when the bytes may be owned.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsString;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.to_bytes();
    /// assert_eq!(os_string, OsString::from_bytes(os_bytes).unwrap());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    /// [`from_cow`]: #method.from_cow
    fn from_bytes<TString>(string: TString) -> Result<Self>
    where
        TString: AsRef<[u8]>;

    /// A convenience method to call either [`from_bytes`] or [`from_vec`],
    /// depending on whether a byte sequence is owned.
    ///
    /// This method can be useful in coordination with
    /// [`OsStrBytes::to_bytes`], since the parameter type matches that
    /// method's return type.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsString;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.to_bytes();
    /// assert_eq!(os_string, OsString::from_cow(os_bytes).unwrap());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    /// [`from_bytes`]: #tymethod.from_bytes
    /// [`from_vec`]: #tymethod.from_vec
    /// [`OsStrBytes::to_bytes`]: trait.OsStrBytes.html#tymethod.to_bytes
    #[inline]
    fn from_cow(string: Cow<'_, [u8]>) -> Result<Self> {
        match string {
            Cow::Borrowed(string) => Self::from_bytes(string),
            Cow::Owned(string) => Self::from_vec(string),
        }
    }

    /// Converts a byte vector into an equivalent platform-native string.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsString;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.clone().into_vec();
    /// assert_eq!(os_string, OsString::from_vec(os_bytes).unwrap());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    fn from_vec(string: Vec<u8>) -> Result<Self>;

    /// Converts a platform-native string into an equivalent byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> io::Result<()> {
    /// let os_string = env::current_exe()?;
    /// println!("{:?}", os_string.into_vec());
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    fn into_vec(self) -> Vec<u8>;
}

impl OsStringBytes for OsString {
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self>
    where
        TString: AsRef<[u8]>,
    {
        imp::os_string_from_bytes(string.as_ref()).map_err(EncodingError)
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self> {
        imp::os_string_from_vec(string).map_err(EncodingError)
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        imp::os_string_into_vec(self)
    }
}

impl OsStringBytes for PathBuf {
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self>
    where
        TString: AsRef<[u8]>,
    {
        OsString::from_bytes(string).map(Into::into)
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self> {
        OsString::from_vec(string).map(Into::into)
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        self.into_os_string().into_vec()
    }
}

mod private {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    pub trait Sealed {}
    impl Sealed for OsStr {}
    impl Sealed for OsString {}
    impl Sealed for Path {}
    impl Sealed for PathBuf {}
}
