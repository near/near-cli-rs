# OsStr Bytes

This crate allows interacting with the data stored internally by [`OsStr`] and
[`OsString`], without resorting to panics or corruption for invalid UTF-8.
Thus, methods can be used that are already defined on [`[u8]`][slice] and
[`Vec<u8>`].

Typically, the only way to losslessly construct [`OsStr`] or [`OsString`] from
a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which requires
the bytes to be valid in UTF-8. However, since this crate makes conversions
directly between the platform encoding and raw bytes, even some strings invalid
in UTF-8 can be converted.

[![GitHub Build Status](https://github.com/dylni/os_str_bytes/workflows/build/badge.svg?branch=master)](https://github.com/dylni/os_str_bytes/actions?query=branch%3Amaster)

## Usage

Add the following lines to your "Cargo.toml" file:

```toml
[dependencies]
os_str_bytes = "2.4"
```

See the [documentation] for available functionality and examples.

## Rust version support

The minimum supported Rust toolchain version depends on the platform:

| `target_os` | Minimum Version |
| --- | --- |
| emscripten (wasm32) | 1.32.0 |
| redox | 1.32.0 |
| unix | 1.32.0 |
| unknown (wasm32) | 1.32.0 |
| wasi | 1.35.0 (all versions) |
| windows | 1.32.0 |

## License

Licensing terms are specified in [COPYRIGHT].

Unless you explicitly state otherwise, any contribution submitted for inclusion
in this crate, as defined in [LICENSE-APACHE], shall be licensed according to
[COPYRIGHT], without any additional terms or conditions.

[COPYRIGHT]: https://github.com/dylni/os_str_bytes/blob/master/COPYRIGHT
[documentation]: https://docs.rs/os_str_bytes
[LICENSE-APACHE]: https://github.com/dylni/os_str_bytes/blob/master/LICENSE-APACHE
[slice]: https://doc.rust-lang.org/std/primitive.slice.html
[`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
[`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
[`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
