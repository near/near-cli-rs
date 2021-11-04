// Copyright (c) 2016
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

//! Idiomatic Rust wrapper for `CommonCrypto`'s `CCKeyDerivationPBKDF` function.

use commoncrypto_sys::{CCKeyDerivationPBKDF, CCPBKDFAlgorithm};

use std::io;

pub use commoncrypto_sys::CCPseudoRandomAlgorithm;

macro_rules! err_from_cckeyderivationpbkdf_retval {
    ($func_name: expr, $val: expr) => {{
        let kind = match $val {
            // kCCParamError is the only one that's specifically noted
            -43000 => io::ErrorKind::InvalidInput,
            _ => io::ErrorKind::Other,
        };

        Err(io::Error::new(kind, format!("{} returned nonzero: {}", $func_name, $val)))
        }}
}

/// Derive a key from a password or passphrase and a salt
pub fn pbkdf2(password: &[u8],
              salt: &[u8],
              prf: CCPseudoRandomAlgorithm,
              rounds: u32,
              key_len: usize)
              -> io::Result<Vec<u8>> {
    let mut pw_derived = vec![0u8; key_len];
    let result = unsafe {
        CCKeyDerivationPBKDF(CCPBKDFAlgorithm::kCCPBKDF2,
                             password.as_ptr(),
                             password.len(),
                             salt.as_ptr(),
                             salt.len(),
                             prf,
                             rounds,
                             pw_derived.as_mut_ptr(),
                             pw_derived.len())
    };

    if result == 0 {
        Ok(pw_derived)
    } else {
        err_from_cckeyderivationpbkdf_retval!("CCKeyDerivationPBKDF", result)
    }
}
