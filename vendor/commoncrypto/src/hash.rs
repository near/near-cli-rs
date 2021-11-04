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

//! Idiomatic Rust wrapper for `CommonCrypto`'s `CCDigestCtx` struct.

use commoncrypto_sys::{CCDigestCreate, CCDigestCtx, CCDigestDestroy, CCDigestFinal,
                       CCDigestGetOutputSizeFromRef, CCDigestReset, CCDigestUpdate};
use std::io;

pub use commoncrypto_sys::CCDigestAlgorithm;

const MAX_DIGEST_SIZE: usize = 64;

macro_rules! err_from_ccdigest_retval{
    ($func_name: expr, $val: expr) => {
        Err(io::Error::new(io::ErrorKind::Other,
                           format!("{} returned nonzero: {}", $func_name, $val)))
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum State {
    Reset,
    Updated,
    Finalized,
}

/// Generates cryptographic hashes.
#[derive(Debug)]
pub struct Hasher {
    ctx: *mut CCDigestCtx,
    state: State,
}

impl Hasher {
    /// Creates a new `Hasher` which will use the given cryptographic `algorithm`.
    pub fn new(algorithm: CCDigestAlgorithm) -> Hasher {
        let ctx: *mut CCDigestCtx;
        unsafe {
            ctx = CCDigestCreate(algorithm);
        }
        Hasher {
            ctx: ctx,
            state: State::Reset,
        }
    }

    fn init(&mut self) {
        match self.state {
            State::Reset => return,
            State::Updated => {
                let _ = self.finish();
            }
            State::Finalized => (),
        }
        unsafe { CCDigestReset(self.ctx) };
        self.state = State::Reset;
    }

    /// Feeds data into the hasher.
    pub fn update(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.state == State::Finalized {
            self.init();
        }
        let result = unsafe { CCDigestUpdate(self.ctx, data.as_ptr() as *mut _, data.len()) };
        if result == 0 {
            self.state = State::Updated;
            Ok(data.len())
        } else {
            err_from_ccdigest_retval!("CCDigestCreate", result)
        }
    }

    /// Finalizes digest operations and produces the digest output.
    pub fn finish(&mut self) -> io::Result<Vec<u8>> {
        if self.state == State::Finalized {
            self.init();
        }
        let expected_len = unsafe { CCDigestGetOutputSizeFromRef(self.ctx) };
        let mut md = vec![0; MAX_DIGEST_SIZE];
        let result = unsafe { CCDigestFinal(self.ctx, md.as_mut_ptr()) };
        if result == 0 {
            self.state = State::Finalized;
            md.truncate(expected_len);
            Ok(md)
        } else {
            err_from_ccdigest_retval!("CCDigestFinal", result)
        }
    }
}

impl io::Write for Hasher {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.update(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Hasher {
    fn drop(&mut self) {
        if self.state != State::Finalized {
            let _ = self.finish();
        }
        unsafe { CCDigestDestroy(self.ctx) }
    }
}
