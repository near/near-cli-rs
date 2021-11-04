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

//! A cryptographic hash generator dependent upon Windows's `CryptoAPI`.
//!
//! Originally based on:
//! https://github.com/rust-lang/cargo/blob/0.10.0/src/cargo/util/sha256.rs
//! which is copyright (c) 2014 The Rust Project Developers under the MIT license.

use super::Algorithm;
use std::io;
use std::ptr;
use winapi::shared::minwindef::DWORD;
use winapi::um::wincrypt::{
    CryptAcquireContextW, CryptCreateHash, CryptDestroyHash, CryptGetHashParam, CryptHashData,
    CryptReleaseContext, ALG_ID, CALG_MD5, CALG_SHA1, CALG_SHA_256, CALG_SHA_512, CRYPT_SILENT,
    CRYPT_VERIFYCONTEXT, HCRYPTHASH, HCRYPTPROV, HP_HASHVAL, PROV_RSA_AES,
};

macro_rules! call {
    ($e:expr) => {{
        if $e == 0 {
            panic!("failed {}: {}", stringify!($e), io::Error::last_os_error())
        }
    }};
}

macro_rules! finish_algorithm {
    ($func_name: ident, $size: ident) => {
        fn $func_name(&mut self) -> Vec<u8> {
            let mut len = $size as u32;
            let mut hash = [0u8; $size];
            call!(unsafe {
                CryptGetHashParam(self.hcrypthash, HP_HASHVAL, hash.as_mut_ptr(), &mut len, 0)
            });
            assert_eq!(len as usize, hash.len());
            hash.to_vec()
        }
    }
}

const MD5_LENGTH: usize = 16;
const SHA1_LENGTH: usize = 20;
const SHA256_LENGTH: usize = 32;
const SHA512_LENGTH: usize = 64;

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
pub struct Hasher {
    alg_id: ALG_ID,
    hcryptprov: HCRYPTPROV,
    hcrypthash: HCRYPTHASH,
}

impl Hasher {
    /// Create a new `Hasher` for the given `Algorithm`.
    pub fn new(algorithm: Algorithm) -> Hasher {
        let mut hcp = 0;
        call!(unsafe {
            CryptAcquireContextW(
                &mut hcp,
                ptr::null(),
                ptr::null(),
                PROV_RSA_AES,
                CRYPT_VERIFYCONTEXT | CRYPT_SILENT,
            )
        });

        let alg_id = match algorithm {
            Algorithm::MD5 => CALG_MD5,
            Algorithm::SHA1 => CALG_SHA1,
            Algorithm::SHA256 => CALG_SHA_256,
            Algorithm::SHA512 => CALG_SHA_512,
        };

        let mut hasher = Hasher {
            alg_id,
            hcryptprov: hcp,
            hcrypthash: 0,
        };

        call!(unsafe {
            CryptCreateHash(
                hasher.hcryptprov,
                hasher.alg_id,
                0,
                0,
                &mut hasher.hcrypthash,
            )
        });
        hasher
    }

    /// Generate a digest from the data written to the `Hasher`.
    pub fn finish(&mut self) -> Vec<u8> {
        match self.alg_id {
            CALG_MD5 => self.finish_md5(),
            CALG_SHA1 => self.finish_sha1(),
            CALG_SHA_256 => self.finish_sha256(),
            CALG_SHA_512 => self.finish_sha512(),
            _ => panic!("Unknown algorithm {}", self.alg_id),
        }
    }

    finish_algorithm!(finish_md5, MD5_LENGTH);
    finish_algorithm!(finish_sha1, SHA1_LENGTH);
    finish_algorithm!(finish_sha256, SHA256_LENGTH);
    finish_algorithm!(finish_sha512, SHA512_LENGTH);
}

impl io::Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        call!(unsafe {
            CryptHashData(
                self.hcrypthash,
                buf.as_ptr() as *mut _,
                buf.len() as DWORD,
                0,
            )
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Hasher {
    fn drop(&mut self) {
        if self.hcrypthash != 0 {
            call!(unsafe { CryptDestroyHash(self.hcrypthash) });
        }
        call!(unsafe { CryptReleaseContext(self.hcryptprov, 0) });
    }
}
