// Copyright (c) 2016 Mark Lee
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

//! A cryptographic hash generator dependent upon OSX's `CommonCrypto`.

use super::Algorithm;
use commoncrypto::hash;
use std::io;

/// Generator of digests using a cryptographic hash function.
///
/// # Examples
///
/// ```rust
/// use crypto_hash::{Algorithm, Hasher};
/// use std::io::Write;
///
/// let mut hasher = Hasher::new(Algorithm::SHA256);
/// hasher.write_all(b"crypto");
/// hasher.write_all(b"-");
/// hasher.write_all(b"hash");
/// let result = hasher.finish();
/// let expected =
///     b"\xfd\x1a\xfb`\"\xcdMG\xc8\x90\x96\x1cS9(\xea\xcf\xe8!\x9f\x1b%$\xf7\xfb*a\x84}\xdf\x8c'"
///     .to_vec();
/// assert_eq!(expected, result)
/// ```
#[derive(Debug)]
pub struct Hasher(hash::Hasher);

impl Hasher {
    /// Create a new `Hasher` for the given `Algorithm`.
    pub fn new(algorithm: Algorithm) -> Hasher {
        let cc_algorithm = match algorithm {
            Algorithm::MD5 => hash::CCDigestAlgorithm::kCCDigestMD5,
            Algorithm::SHA1 => hash::CCDigestAlgorithm::kCCDigestSHA1,
            Algorithm::SHA256 => hash::CCDigestAlgorithm::kCCDigestSHA256,
            Algorithm::SHA512 => hash::CCDigestAlgorithm::kCCDigestSHA512,
        };

        Hasher(hash::Hasher::new(cc_algorithm))
    }

    /// Generate a digest from the data written to the `Hasher`.
    pub fn finish(&mut self) -> Vec<u8> {
        let Hasher(ref mut hasher) = *self;
        match hasher.finish() {
            Ok(digest) => digest,
            Err(error) => panic!("CommonCrypto error: {}", error),
        }
    }
}

impl io::Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let Hasher(ref mut hasher) = *self;
        hasher.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let Hasher(ref mut hasher) = *self;
        hasher.flush()
    }
}
