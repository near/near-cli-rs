# Contributing to `crypto-hash`

`crypto-hash` is a part of the Rust ecosystem. As such, all contributions to this project follow the
[Rust language's code of conduct](https://www.rust-lang.org/conduct.html) where appropriate.

This project is hosted at [GitHub](https://github.com/malept/crypto-hash). Both pull requests and
issues of many different kinds are accepted.

## Filing Issues

Issues include bugs, questions, feedback, and feature requests. Before you file a new issue, please
make sure that your issue has not already been filed by someone else.

### Filing Bugs

When filing a bug, please include the following information:

* Operating system and version. If on Linux, please also include the distribution name.
* System architecture. Examples include: x86-64, x86, and ARMv7.
* Rust version that compiled `crypto-hash`.
* The version (and/or git revision) of `crypto-hash`.
* A detailed list of steps to reproduce the bug. A minimal testcase would be very helpful,
  if possible.
* If there any any error messages in the console, copying them in the bug summary will be
  very helpful.

## Adding a new implementation

If you are requesting or adding a new library source for hash algorithms, please make sure that it
supports all of the existing algorithms. For example, while the creator of this project supports the
efforts of the team writing LibreSSL, it does not support the MD5 algorithm.

## Adding a new hash algorithm

If you are requesting or adding a wrapper for a new hash algorithm, please make sure that it is
available in all of the supported implementations listed in the README.

## Filing Pull Requests

Here are some things to keep in mind as you file a pull request to fix a bug, add a new feature,
etc.:

* Travis CI (for Linux and OS X) and AppVeyor (for Windows) are used to make sure that the project
  builds as expected on the supported platforms, using the current stable and beta versions of Rust.
  Make sure the testsuite passes locally by running `cargo test`.
* Unless it's impractical, please write tests for your changes. This will help spot regressions
  much easier.
* If your PR changes the behavior of an existing feature, or adds a new feature, please add/edit
  the `rustdoc` inline documentation.
* Please ensure that your changes follow the [rustfmt](https://github.com/rust-lang-nursery/rustfmt)
  coding standard, and do not produce any warnings when running the
  [clippy](https://github.com/Manishearth/rust-clippy) linter.
* If you are contributing a nontrivial change, please add an entry to `NEWS.md`. The format is
  similar to the one described at [Keep a Changelog](http://keepachangelog.com/).
* Please make sure your commits are rebased onto the latest commit in the master branch, and that
  you limit/squash the number of commits created to a "feature"-level. For instance:

bad:

```
commit 1: add foo algorithm
commit 2: run rustfmt
commit 3: add test
commit 4: add docs
commit 5: add bar
commit 6: add test + docs
```

good:

```
commit 1: add foo algorithm
commit 2: add bar
```

If you are continuing the work of another person's PR and need to rebase/squash, please retain the
attribution of the original author(s) and continue the work in subsequent commits.
