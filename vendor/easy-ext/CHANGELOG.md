# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.2.6] - 2021-01-19

- [Support specifying visibility at impl-level.](https://github.com/taiki-e/easy-ext/pull/25)

## [0.2.5] - 2021-01-05

- Exclude unneeded files from crates.io.

## [0.2.4] - 2020-12-29

- Documentation improvements.

## [0.2.3] - 2020-08-24

- [Documentation (`#[doc]` attributes) is now generated only for trait definitions.](https://github.com/taiki-e/easy-ext/pull/23) Previously it generated for both trait definition and trait implementation. See [#20](https://github.com/taiki-e/easy-ext/issues/20) for more details.

## [0.2.2] - 2020-07-22

- [Fix `unused_attributes` lint in generated code.](https://github.com/taiki-e/easy-ext/pull/22)

- Diagnostic improvements.

## [0.2.1] - 2020-07-11

- Documentation improvements.

## [0.2.0] - 2020-04-22

- [`#[ext]` no longer adds type parameter, which is equivalent to `Self`, to the trait's generics.](https://github.com/taiki-e/easy-ext/pull/15) See [#11](https://github.com/taiki-e/easy-ext/issues/11) for more details.

## [0.1.8] - 2020-04-20

- Documentation improvements.

## [0.1.7] - 2020-04-20

- [Supported unnamed extension trait.](https://github.com/taiki-e/easy-ext/pull/9)

## [0.1.6] - 2019-10-12

- [Improved error messages related to visibility.](https://github.com/taiki-e/easy-ext/pull/5)

## [0.1.5] - 2019-08-15

- Updated `syn` and `quote` to 1.0.

## [0.1.4] - 2019-03-10

- Updated minimum `syn` version to 0.15.29.

## [0.1.3] - 2019-02-21

- Removed `inline` attributes on trait method side. It can avoid [`clippy::inline_fn_without_body`](https://rust-lang.github.io/rust-clippy/master/index.html#inline_fn_without_body) by this.

## [0.1.2] - 2019-02-21

- Used `#[allow(patterns_in_fns_without_body)]` to generated extension trait.

- Fixed some bugs related to generics.

## [0.1.1] - 2019-02-21

**Note: This release has been yanked.**

- Fixed an error related to generics.

## [0.1.0] - 2019-02-20

**Note: This release has been yanked.**

Initial release

[Unreleased]: https://github.com/taiki-e/easy-ext/compare/v0.2.6...HEAD
[0.2.6]: https://github.com/taiki-e/easy-ext/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/taiki-e/easy-ext/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/taiki-e/easy-ext/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/taiki-e/easy-ext/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/taiki-e/easy-ext/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/taiki-e/easy-ext/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/taiki-e/easy-ext/compare/v0.1.8...v0.2.0
[0.1.8]: https://github.com/taiki-e/easy-ext/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/taiki-e/easy-ext/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/taiki-e/easy-ext/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/taiki-e/easy-ext/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/taiki-e/easy-ext/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/taiki-e/easy-ext/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/taiki-e/easy-ext/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/easy-ext/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/easy-ext/releases/tag/v0.1.0
