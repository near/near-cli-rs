// Copyright (c) 2015, 2016, 2017 Mark Lee
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! A set of [cryptographic hash
//! functions](https://en.wikipedia.org/wiki/Cryptographic_hash_function) provided by the operating
//! system, when available.
//!
//! The purpose of this crate is to provide access to hash algorithms with as few dependencies as
//! possible. This means that when possible, the library uses the hashing functions that are
//! provided by the given operating system's bundled cryptographic libraries.
//!
//! # Supported Implementations
//!
//! By operating system:
//!
//! * Windows: `CryptoAPI`
//! * Mac OS X: `CommonCrypto`
//! * Linux/BSD/etc.: `OpenSSL`
//!
//! # Supported Algorithms
//!
//! * MD5
//! * SHA1
//! * SHA256
//! * SHA512

#![warn(missing_docs)]

#[cfg(any(target_os = "macos", target_os = "ios"))]
extern crate commoncrypto;
extern crate hex;
#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
extern crate openssl;
#[cfg(target_os = "windows")]
extern crate winapi;

use std::io::Write;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[path = "imp/commoncrypto.rs"]
mod imp;
#[cfg(target_os = "windows")]
#[path = "imp/cryptoapi.rs"]
mod imp;
#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
#[path = "imp/openssl.rs"]
mod imp;

mod test;

pub use imp::Hasher;

/// Available cryptographic hash functions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// Popular message digest algorithm, only available for backwards compatibility purposes.
    MD5,
    /// SHA-1 algorithm from NIST FIPS, only available for backwards compatibility purposes.
    SHA1,
    /// SHA-2 family algorithm (256 bits).
    SHA256,
    /// SHA-2 family algorithm (512 bits).
    SHA512,
}

/// Helper function for `Hasher` which generates a cryptographic digest from the given
/// data and algorithm.
///
/// # Examples
///
/// ```rust
/// use crypto_hash::{Algorithm, digest};
///
/// let data = b"crypto-hash";
/// let result = digest(Algorithm::SHA256, data);
/// let expected =
///     b"\xfd\x1a\xfb`\"\xcdMG\xc8\x90\x96\x1cS9(\xea\xcf\xe8!\x9f\x1b%$\xf7\xfb*a\x84}\xdf\x8c'"
///     .to_vec();
/// assert_eq!(expected, result)
/// ```
pub fn digest(algorithm: Algorithm, data: &[u8]) -> Vec<u8> {
    let mut hasher = imp::Hasher::new(algorithm);
    hasher.write_all(data).expect("Could not write hash data");
    hasher.finish()
}

/// Helper function for `Hasher` which generates a cryptographic digest serialized in
/// hexadecimal from the given data and algorithm.
///
/// # Examples
///
/// ```rust
/// use crypto_hash::{Algorithm, hex_digest};
///
/// let data = b"crypto-hash";
/// let result = hex_digest(Algorithm::SHA256, data);
/// let expected = "fd1afb6022cd4d47c890961c533928eacfe8219f1b2524f7fb2a61847ddf8c27";
/// assert_eq!(expected, result)
/// ```
pub fn hex_digest(algorithm: Algorithm, data: &[u8]) -> String {
    hex::encode(digest(algorithm, data))
}
