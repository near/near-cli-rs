# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/near/near-cli-rs/compare/v0.3.1...v0.3.2) - 2023-04-11

### Added
- Show contract function call result and make it usable in scripts by only writing the result to stdout, and everything else will be printed to stderr (#177)
- Use optimisticly latest nonce for the access key instead of the final one to avoid errors running commands one after the other (#176)

### Fixed
- fixed printing about saving the access key (#173)

### Other
- Added GitHub Actions installation instructions

## [0.3.1](https://github.com/near/near-cli-rs/compare/v0.3.0...v0.3.1) - 2023-04-06

### Other
- Use custom GITHUB_TOKEN to be able to trigger follow up CI jobs
- make a git tag with Release-plz, so it triggers binary release pipeline (#170)

## [0.3.0] - 2023-04-06

* Renamed `near-cli` binary to `near` as it has special handlers for the commands of near CLI JS, and can be used as a replacement to near CLI JS that will guide users on how to use the new commands when they type the old commands.
* Improved continuous integration pipelines to streamline releases (each push to `master` branch will trigger a pipeline that will create a PR suggesting to cut a new release, and once the version is ticked, crate will be published and tagged, and then binary release pipeline will kick in)
