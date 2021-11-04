# `crypto-hash`

[![Linux/OS X Status](https://travis-ci.org/malept/crypto-hash.svg?branch=master)](https://travis-ci.org/malept/crypto-hash)
[![Windows status](https://ci.appveyor.com/api/projects/status/xwc9nb4633b5n67r/branch/master?svg=true)](https://ci.appveyor.com/project/malept/crypto-hash)
[![Crates.io](https://img.shields.io/crates/v/crypto-hash.svg?maxAge=2592000)](https://crates.io/crates/crypto-hash)

`crypto-hash` is a Rust wrapper around OS-level implementations of cryptographic hash functions.

The purpose of this crate is to provide access to hash algorithms with as few dependencies as
possible. This means that when possible, the library uses the hashing functions that are provided by
the given operating system's bundled cryptographic libraries.

## Supported Implementations

By operating system:

* Windows: CryptoAPI
* OS X: [CommonCrypto](https://crates.io/crates/commoncrypto)
* Linux/BSD/etc.: [OpenSSL](https://crates.io/crates/openssl)

## Supported Algorithms

* MD5
* SHA1
* SHA256
* SHA512

## Usage

Add `crypto-hash` to your project's `Cargo.toml`. For more details, consult the
[Cargo guide](http://doc.crates.io/guide.html#adding-dependencies).

Example:

```rust
use crypto_hash::{Algorithm, hex_digest};

let digest = hex_digest(Algorithm::SHA256, b"crypto-hash");
```

For more examples, consult the [documentation](https://malept.github.io/crypto-hash/).

## [Release Notes](https://github.com/malept/crypto-hash/blob/master/NEWS.md)

## [Contributing](https://github.com/malept/crypto-hash/blob/master/CONTRIBUTING.md)

## Acknowledgements

This crate was inspired by [rust-native-tls](https://github.com/sfackler/rust-native-tls) and
[crypto-bench](https://github.com/briansmith/crypto-bench).

## Legal

`crypto-hash` is copyrighted under the terms of the MIT license. See LICENSE for details.
