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

//! Low-level bindings to OSX/macOS/iOS's `CommonCrypto` library.

#![warn(missing_docs)]

extern crate libc;

use libc::{c_int, c_uint};

/// Total number of operations.
const MD5_CBLOCK: usize = 64;
/// Number of operations per round.
const MD5_LBLOCK: usize = MD5_CBLOCK / 4;
/// Number of bytes for an MD5 hash.
pub const MD5_DIGEST_LENGTH: usize = 16;

const SHA_LBLOCK: usize = 16;
/// Number of bytes for an SHA1 hash.
pub const SHA1_DIGEST_LENGTH: usize = 20;
/// Number of bytes for an SHA256 hash.
pub const SHA256_DIGEST_LENGTH: usize = 32;
/// Number of bytes for an SHA384 hash.
pub const SHA384_DIGEST_LENGTH: usize = 48;
/// Number of bytes for an SHA512 hash.
pub const SHA512_DIGEST_LENGTH: usize = 64;

/// Struct used to generate MD5 hashes.
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct CC_MD5_CTX {
    A: c_uint,
    B: c_uint,
    C: c_uint,
    D: c_uint,
    Nl: c_uint,
    Nh: c_uint,
    data: [c_uint; MD5_LBLOCK],
    num: c_uint,
}

/// Struct used to generate SHA1 hashes.
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct CC_SHA_CTX {
    h0: c_uint,
    h1: c_uint,
    h2: c_uint,
    h3: c_uint,
    h4: c_uint,
    Nl: c_uint,
    Nh: c_uint,
    data: [c_uint; SHA_LBLOCK],
    num: c_uint,
}

macro_rules! cc_sha2_struct {
    ($ctx_name: ident, $ty: ty) => {
        /// Struct used to generate SHA2 hashes with the given bits.
        #[allow(non_camel_case_types, non_snake_case)]
        #[derive(Clone, Debug, Default, PartialEq)]
        #[repr(C)]
        pub struct $ctx_name {
            count: [$ty; 2],
            hash: [$ty; 8],
            wbuf: [$ty; 16],
        }
    }
}

cc_sha2_struct!(CC_SHA256_CTX, u32);
cc_sha2_struct!(CC_SHA512_CTX, u64);

/// Digest algorithm used in `CCDigest*()` functions.
#[repr(C)]
pub enum CCDigestAlgorithm {
    /// No digest algorithm
    kCCDigestNone = 0,
    /// MD2
    kCCDigestMD2 = 1,
    /// MD4
    kCCDigestMD4 = 2,
    /// MD5
    kCCDigestMD5 = 3,
    /// RIPEMD-128
    kCCDigestRMD128 = 4,
    /// RIPEMD-160
    kCCDigestRMD160 = 5,
    /// RIPEMD-256
    kCCDigestRMD256 = 6,
    /// RIPEMD-320
    kCCDigestRMD320 = 7,
    /// SHA1
    kCCDigestSHA1 = 8,
    /// SHA224
    kCCDigestSHA224 = 9,
    /// SHA256
    kCCDigestSHA256 = 10,
    /// SHA384
    kCCDigestSHA384 = 11,
    /// SHA512
    kCCDigestSHA512 = 12,
    /// Skein, 128 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein128 = 13,
    /// Skein, 160 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein160 = 14,
    /// Skein, 224 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein224 = 16,
    /// Skein, 256 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein256 = 17,
    /// Skein, 384 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein384 = 18,
    /// Skein, 512 bit (Deprecated in iPhoneOS 6.0 and MacOSX10.9)
    kCCDigestSkein512 = 19,
}

const CC_DIGEST_SIZE: usize = 1032;

/// Context used in `CCDigest*()` functions.
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct CCDigestCtx {
    context: [u8; CC_DIGEST_SIZE],
}

/// Algorithm for use with `CCKeyDerivationPBKDF()`.
#[repr(C)]
pub enum CCPBKDFAlgorithm {
    /// PBKDF2
    kCCPBKDF2 = 2,
}

/// Pseudo-random algorithm to use with `CCKeyDerivationPBKDF()`.
#[repr(C)]
pub enum CCPseudoRandomAlgorithm {
    /// SHA-1
    kCCPRFHmacAlgSHA1 = 1,
    /// SHA-224
    kCCPRFHmacAlgSHA224 = 2,
    /// SHA-256
    kCCPRFHmacAlgSHA256 = 3,
    /// SHA-384
    kCCPRFHmacAlgSHA384 = 4,
    /// SHA-512
    kCCPRFHmacAlgSHA512 = 5,
}

extern "C" {
    /// Initializes MD5 hasher. See `man 3cc CC_MD5` for details.
    pub fn CC_MD5_Init(ctx: *mut CC_MD5_CTX) -> c_int;
    /// Appends data to be hashed. See `man 3cc CC_MD5` for details.
    pub fn CC_MD5_Update(ctx: *mut CC_MD5_CTX, data: *const u8, n: usize) -> c_int;
    /// Generates MD5 hash. See `man 3cc CC_MD5` for details.
    pub fn CC_MD5_Final(md: *mut u8, ctx: *mut CC_MD5_CTX) -> c_int;
    /// Initializes SHA1 hasher. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA1_Init(ctx: *mut CC_SHA_CTX) -> c_int;
    /// Appends data to be hashed. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA1_Update(ctx: *mut CC_SHA_CTX, data: *const u8, n: usize) -> c_int;
    /// Generates SHA1 hash. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA1_Final(md: *mut u8, ctx: *mut CC_SHA_CTX) -> c_int;
    /// Initializes SHA256 hasher. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA256_Init(ctx: *mut CC_SHA256_CTX) -> c_int;
    /// Appends data to be hashed. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA256_Update(ctx: *mut CC_SHA256_CTX, data: *const u8, n: usize) -> c_int;
    /// Generates SHA256 hash. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA256_Final(md: *mut u8, ctx: *mut CC_SHA256_CTX) -> c_int;
    /// Initializes SHA384 hasher. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA384_Init(ctx: *mut CC_SHA512_CTX) -> c_int;
    /// Appends data to be hashed. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA384_Update(ctx: *mut CC_SHA512_CTX, data: *const u8, n: usize) -> c_int;
    /// Generates SHA384 hash. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA384_Final(md: *mut u8, ctx: *mut CC_SHA512_CTX) -> c_int;
    /// Initializes SHA512 hasher. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA512_Init(ctx: *mut CC_SHA512_CTX) -> c_int;
    /// Appends data to be hashed. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA512_Update(ctx: *mut CC_SHA512_CTX, data: *const u8, n: usize) -> c_int;
    /// Generates SHA512 hash. See `man 3cc CC_SHA` for details.
    pub fn CC_SHA512_Final(md: *mut u8, ctx: *mut CC_SHA512_CTX) -> c_int;
    /// Generic digest hasher.
    pub fn CCDigest(algorithm: CCDigestAlgorithm,
                    data: *const u8,
                    length: usize,
                    output: *mut u8)
                    -> c_int;
    /// Allocate and initialize a `CCDigestCtx` for a digest.
    pub fn CCDigestCreate(algorithm: CCDigestAlgorithm) -> *mut CCDigestCtx;
    /// Continue to digest data. Returns `0` on success.
    pub fn CCDigestUpdate(ctx: *mut CCDigestCtx, data: *const u8, length: usize) -> c_int;
    /// Conclude digest operations and produce the digest output. Returns `0` on success.
    pub fn CCDigestFinal(ctx: *mut CCDigestCtx, output: *mut u8) -> c_int;
    /// Clear and free a `CCDigestCtx`.
    pub fn CCDigestDestroy(ctx: *mut CCDigestCtx);
    /// Clear and re-initialize a `CCDigestCtx` for the same algorithm.
    pub fn CCDigestReset(ctx: *mut CCDigestCtx);
    /// Produce the digest output result for the bytes currently processed. Returns `0` on success.
    pub fn CCDigestGetDigest(ctx: *mut CCDigestCtx, output: *mut u8) -> c_int;
    /// Provides the block size of the digest algorithm. Returns `0` on failure.
    pub fn CCDigestGetBlockSize(algorithm: CCDigestAlgorithm) -> usize;
    /// Provides the digest output size of the digest algorithm. Returns `0` on failure.
    pub fn CCDigestGetOutputSize(algorithm: CCDigestAlgorithm) -> usize;
    /// Provides the block size of the digest algorithm. Returns `0` on failure.
    pub fn CCDigestGetBlockSizeFromRef(ctx: *mut CCDigestCtx) -> usize;
    /// Provides the digest output size of the digest algorithm. Returns `0` on failure.
    pub fn CCDigestGetOutputSizeFromRef(ctx: *mut CCDigestCtx) -> usize;

    /// Derive a key from a user-supplied password via PBKDF2.
    pub fn CCKeyDerivationPBKDF(algorithm: CCPBKDFAlgorithm,
                                password: *const u8,
                                passwordLen: usize,
                                salt: *const u8,
                                saltLen: usize,
                                prf: CCPseudoRandomAlgorithm,
                                rounds: u32,
                                derivedKey: *mut u8,
                                derivedKeyLen: usize)
                                -> c_int;
}
