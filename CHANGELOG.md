# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.6](https://github.com/near/near-cli-rs/compare/v0.7.5...v0.7.6) - 2023-12-21

### Other
- Removed excessive step during transaction status view to improve UX ([#280](https://github.com/near/near-cli-rs/pull/280))
- Updated reconstruct-transaction command ([#281](https://github.com/near/near-cli-rs/pull/281))

## [0.7.5](https://github.com/near/near-cli-rs/compare/v0.7.4...v0.7.5) - 2023-12-19

### Added
- Improved self-update UX with more details ([#274](https://github.com/near/near-cli-rs/pull/274))

### Fixed
- Display NEAR token amounts precisely ([#278](https://github.com/near/near-cli-rs/pull/278))

### Other
- Updated the guide around the usage of system keychain on Linux, Windows, and macOS ([#277](https://github.com/near/near-cli-rs/pull/277))
- Added explicit installation instructions to README.md

## [0.7.4](https://github.com/near/near-cli-rs/compare/v0.7.3...v0.7.4) - 2023-12-06

### Added
- Removed the need for an additional network selection step if there is only one network connection in the config file ([#272](https://github.com/near/near-cli-rs/pull/272))
- Added the ability to interactively select access keys to remove from the list of public keys ([#269](https://github.com/near/near-cli-rs/pull/269))

### Other
- Cleaned up duplicative CI checks and renamed 'master' to 'main' default branch
- Automate publishing near-cli-rs to npmjs for `npx near-cli-rs` to use the latest released binary release by default ([#273](https://github.com/near/near-cli-rs/pull/273))

## [0.7.3](https://github.com/near/near-cli-rs/compare/v0.7.2...v0.7.3) - 2023-12-02

### Added
- Added support for blind signing with Ledger [requires updated Ledger app that is not yet published] ([#259](https://github.com/near/near-cli-rs/pull/259))
- New command to reconstruct NEAR CLI command from a historical transaction ([#266](https://github.com/near/near-cli-rs/pull/266))

### Other
- Addressed all default cargo clippy warnings ([#263](https://github.com/near/near-cli-rs/pull/263))

## [0.7.2](https://github.com/near/near-cli-rs/compare/v0.7.1...v0.7.2) - 2023-11-21

### Added
- Show hash-to-sign when using the sign_later transaction signature option ([#261](https://github.com/near/near-cli-rs/pull/261))

## [0.7.1](https://github.com/near/near-cli-rs/compare/v0.7.0...v0.7.1) - 2023-11-17

### Added
- add `--offline` for `sign-with-ledger` option ([#260](https://github.com/near/near-cli-rs/pull/260))
- Updated legacy command compatibility for near-cli (JS) for dev-deploy, validators, and staking commands ([#256](https://github.com/near/near-cli-rs/pull/256))

## [0.7.0](https://github.com/near/near-cli-rs/compare/v0.6.2...v0.7.0) - 2023-10-31

### Added
- New command: staking - delegation ([#227](https://github.com/near/near-cli-rs/pull/227))

### Other
- Refactored NEAR tokens usages to use a strictly typed near-token crate ([#253](https://github.com/near/near-cli-rs/pull/253))

## [0.6.2](https://github.com/near/near-cli-rs/compare/v0.6.1...v0.6.2) - 2023-10-17

### Added
- Exposed some of the functions to use "manage-profile" in bos-cli-rs ([#249](https://github.com/near/near-cli-rs/pull/249))
- Exposed subcommands related to "deploy" to reuse in cargo-near ([#247](https://github.com/near/near-cli-rs/pull/247))

## [0.6.1](https://github.com/near/near-cli-rs/compare/v0.6.0...v0.6.1) - 2023-10-09

### Added
- Added a new command to manage BOS profile in SocialDB ([#231](https://github.com/near/near-cli-rs/pull/231))
- Provide a relevant faucet error message when helper API server returns an error ([#243](https://github.com/near/near-cli-rs/pull/243))

### Other
- Exposed sponsor_by_faucet_service module to re-use in "cargo-near" ([#246](https://github.com/near/near-cli-rs/pull/246))

## [0.6.0](https://github.com/near/near-cli-rs/compare/v0.5.2...v0.6.0) - 2023-09-28

### Added
- New command export-account ([#226](https://github.com/near/near-cli-rs/pull/226))
- [**breaking**] Added system keychain support for windows & linux, so now all major desktop operating systems are supported! ([#232](https://github.com/near/near-cli-rs/pull/232))
- order networks selection based on the selected account id (bubble up more relevant networks) ([#225](https://github.com/near/near-cli-rs/pull/225))

### Fixed
- CLI must return a non-zero exit code when function call (as-transaction) fails ([#238](https://github.com/near/near-cli-rs/pull/238))
- legacy view-function call with --base64 was not recognized ([#237](https://github.com/near/near-cli-rs/pull/237))
- self update is now pointing to the right archive name ([#234](https://github.com/near/near-cli-rs/pull/234))

### Other
- Upgraded cargo-dist to 0.3.0 version to enable MSI Windows installer, and binary artifacts on every PR ([#241](https://github.com/near/near-cli-rs/pull/241))
- Switch to near-gas crate  ([#240](https://github.com/near/near-cli-rs/pull/240))
- New command to view contract storage state ([#239](https://github.com/near/near-cli-rs/pull/239))

## [0.5.2](https://github.com/near/near-cli-rs/compare/v0.5.1...v0.5.2) - 2023-08-17

### Added
- Select your account when prompted interactively ([#224](https://github.com/near/near-cli-rs/pull/224))
- Allow to specify a custom Web Wallet URL when importing account (default to MyNearWallet) ([#218](https://github.com/near/near-cli-rs/pull/218))

### Other
- Upgrade cargo-dist to 0.1.0 release ([#229](https://github.com/near/near-cli-rs/pull/229))

## [0.5.1](https://github.com/near/near-cli-rs/compare/v0.5.0...v0.5.1) - 2023-06-07

### Added
- New `transaction sign-transaction` command [useful in combination with `... sign-later` and `transaction send-signed-transaction` commands] ([#215](https://github.com/near/near-cli-rs/pull/215))

### Other
- Upgraded NEAR crates to 0.17.0 release ([#216](https://github.com/near/near-cli-rs/pull/216))

## [0.5.0](https://github.com/near/near-cli-rs/compare/v0.4.3...v0.5.0) - 2023-06-05

### Added
- New offline mode allows to prepare transactions on devices that are not connected to the Internet ([#209](https://github.com/near/near-cli-rs/pull/209))

### Fixed
- Add support for no-args view-function calls for legacy JS CLI `view` command ([#213](https://github.com/near/near-cli-rs/pull/213))

## [0.4.3](https://github.com/near/near-cli-rs/compare/v0.4.2...v0.4.3) - 2023-06-02

### Added
- New command to send a signed transaction [potentially constructed offline] ([#206](https://github.com/near/near-cli-rs/pull/206))
- Extended access-key deletion with an option to delete multiple keys in a single transaction ([#207](https://github.com/near/near-cli-rs/pull/207))

### Other
- Updated dependencies to the most recent versions

## [0.4.2](https://github.com/near/near-cli-rs/compare/v0.4.1...v0.4.2) - 2023-05-26

### Added
- Added Json type ([#203](https://github.com/near/near-cli-rs/pull/203))

## [0.4.1](https://github.com/near/near-cli-rs/compare/v0.4.0...v0.4.1) - 2023-05-22

### Fixed
- Added extra space at the beginning of a line in interactive queries (#196)

### Other
- Added a guide on `send-meta-transaction` (#192)

## [0.4.0](https://github.com/near/near-cli-rs/compare/v0.3.5...v0.4.0) - 2023-05-02

### Added
- Meta-Transactions support (#189)
- Support for adding key from Ledger hardware wallet (#188)

### Fixed
- fixed call function with non-JSON arguments being incorrectly displayed as `null` (#187)
- pass right token to release-plz action (#185)

## [0.3.5](https://github.com/near/near-cli-rs/compare/v0.3.4...v0.3.5) - 2023-04-21

### Fixed
- Fixed self-update to use the proper archive name generated by cargo-dist

## [0.3.4](https://github.com/near/near-cli-rs/compare/v0.3.3...v0.3.4) - 2023-04-20

### Other
- Enable self-update on CI and NPM installer for binary releases (#183)
- release v0.3.3 (#182)

## [0.3.3](https://github.com/near/near-cli-rs/compare/v0.3.2...v0.3.3) - 2023-04-20

### Added
- Added support for Contract Storage Management Standard (#179)

### Other
- update release-plz-action to v0.5 (#180)

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
