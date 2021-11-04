# `crypto-hash`: Changes by Version

## [Unreleased](https://github.com/malept/crypto-hash/compare/v0.3.3...master)

## [0.3.3] - 2018-12-20

[0.3.3]: https://github.com/malept/crypto-hash/compare/v0.3.2...v0.3.3

### Changed

* Revert API change (#6)

## [0.3.2] - 2018-12-20

[0.3.2]: https://github.com/malept/crypto-hash/compare/v0.3.1...v0.3.2

**Note: This release was yanked from Cargo due to #6.**

### Added

* iOS support

## [0.3.1] - 2018-02-14

[0.3.1]: https://github.com/malept/crypto-hash/compare/v0.3.0...v0.3.1

### Changed

* Upgrade to `openssl` 0.10.x (#1)
* Upgrade to `winapi` 0.3.x

## [0.3.0] - 2017-06-18

[0.3.0]: https://github.com/malept/crypto-hash/compare/v0.2.1...v0.3.0

### Changed

* Upgrade to `commoncrypto` 0.2.x
* Function signatures for `digest` and `hex_digest` changed to use `&[u8]`, per Clippy

## [0.2.1] - 2016-12-12

[0.2.1]: https://github.com/malept/crypto-hash/compare/v0.2.0...v0.2.1

### Changed

* Move CommonCrypto implementation to its own crate

## [0.2.0] - 2016-11-06

[0.2.0]: https://github.com/malept/crypto-hash/compare/v0.1.0...v0.2.0

### Added

* SHA-1 algorithm

### Changed

* Upgrade rust-openssl to 0.9

## [0.1.0] - 2016-06-26

[0.1.0]: https://github.com/malept/crypto-hash/releases/tag/v0.1.0

This release signifies the minimum amount of algorithms and implementations necessary for
[HTTP digest authentication](https://tools.ietf.org/html/rfc7616).

### Added

Algorithms:

* MD5
* SHA256
* SHA512

Implementations:

* CommonCrypto (OS X)
* CryptoAPI (Windows)
* OpenSSL (Linux/BSD/etc.)
