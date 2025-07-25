# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.22.0](https://github.com/near/near-cli-rs/compare/v0.21.0...v0.22.0) - 2025-07-21

### Added

- New "message sign-nep413" command - NEP-413 offchain messages signing ([#507](https://github.com/near/near-cli-rs/pull/507))

### Fixed

- Gracefully handle (Started, NotStarted) transaction status ([#510](https://github.com/near/near-cli-rs/pull/510))

### Other

- Removed dj8yfo from CODEOWNERS

## [0.21.0](https://github.com/near/near-cli-rs/compare/v0.20.0...v0.21.0) - 2025-07-08

### Added

- Improved support for importing implicit accounts ([#498](https://github.com/near/near-cli-rs/pull/498))
- Added a command to confirm locally that the deployed contract matches the source code it was built from ([#480](https://github.com/near/near-cli-rs/pull/480))
- Add support for submitting general DAO proposals ([#492](https://github.com/near/near-cli-rs/pull/492))

### Fixed

- Fixed typos in the default CLI config ([#501](https://github.com/near/near-cli-rs/pull/501))
- Fixed error RpcError: [missing field disable_9393_fix] ([#502](https://github.com/near/near-cli-rs/pull/502))
- use re-rexported version of `indicatif` module, needed for `tracing-indicatif` ([#503](https://github.com/near/near-cli-rs/pull/503))
- `clippy::result_large_err` warning ([#488](https://github.com/near/near-cli-rs/pull/488))

### Other

- Added `verify_contract`/`inspect_contract` feauture gates ([#504](https://github.com/near/near-cli-rs/pull/504))
- Pinned binary releases to use ubuntu-22.04 ([#506](https://github.com/near/near-cli-rs/pull/506))
- Fixed linting errors - non-inlined formatting syntax ([#499](https://github.com/near/near-cli-rs/pull/499))
- fix some typos ([#494](https://github.com/near/near-cli-rs/pull/494))
- simplified action struct ([#493](https://github.com/near/near-cli-rs/pull/493))
- added more RPCs (FAST NEAR and Lava) to the default config generation ([#491](https://github.com/near/near-cli-rs/pull/491))
- introduced `BoxedJsonRpcResult` type ([#490](https://github.com/near/near-cli-rs/pull/490))
- fixed shell configuration for command history ([#487](https://github.com/near/near-cli-rs/pull/487))

## [0.20.0](https://github.com/near/near-cli-rs/compare/v0.19.0...v0.20.0) - 2025-05-13

### Added

- display global contracts info on account details query ([#483](https://github.com/near/near-cli-rs/pull/483))
- extended action builder with new released items ([#457](https://github.com/near/near-cli-rs/pull/457))
- deploy and use global contracts from UI excluding action builder. ([#456](https://github.com/near/near-cli-rs/pull/456))
- Do not require to explicitly provide public key during signing transactions with a plaintext private key (we can always extract it from the private key) ([#468](https://github.com/near/near-cli-rs/pull/468))
- Added new command "get public key from plaintext private key" ([#466](https://github.com/near/near-cli-rs/pull/466))
- Use free ARM64 GitHub Action runners for pre-built binaries ([#454](https://github.com/near/near-cli-rs/pull/454))

### Fixed

- Fixed returning the result of a function call ([#462](https://github.com/near/near-cli-rs/pull/462))
- Fixed canceling the CLI operation in retry prompts ([#478](https://github.com/near/near-cli-rs/pull/478))
- Fixed cli command `near tokens send-ft` without `memo` ([#474](https://github.com/near/near-cli-rs/pull/474))
- allow forks to leverage transfer-to-project workflow ([#464](https://github.com/near/near-cli-rs/pull/464))

### Other

- removed unsupported reconstruction for DeployGlobalContract ([#484](https://github.com/near/near-cli-rs/pull/484))
- [**breaking**] updates near-* dependencies to 0.30 release ([#481](https://github.com/near/near-cli-rs/pull/481))
- fixed reconstruction of transaction from contract deploy ([#476](https://github.com/near/near-cli-rs/pull/476))
- Refactored 'send-ft memo' command from using Option<String> to just String ([#477](https://github.com/near/near-cli-rs/pull/477))
- Replaced Linux x86-64 ubuntu-20.04 build environment with ubuntu-22.04 as 20.04 reached its end of life, so minimal supported glibc for pre-built binaries is now 2.35 - Linux users with older glibc will have to install CLIs from the source code or update their OS ([#470](https://github.com/near/near-cli-rs/pull/470))
- updated GUIDE ([#469](https://github.com/near/near-cli-rs/pull/469))
- Updated CI secret name in the devtools pipeline ([#460](https://github.com/near/near-cli-rs/pull/460))

## [0.19.0](https://github.com/near/near-cli-rs/compare/v0.18.0...v0.19.0) - 2025-03-12

### Added

- Improved usability of NEAR CLI in scripts ([#445](https://github.com/near/near-cli-rs/pull/445))
- Added "send all" option to fungible tokens ([#440](https://github.com/near/near-cli-rs/pull/440))
- let users specify a memo when sending ft
- A new `--quiet` flag to suppress noisy output in scripting scenarios ([#441](https://github.com/near/near-cli-rs/pull/441))

### Fixed

- Fixed information about successful transfer of "send all" ft tokens ([#447](https://github.com/near/near-cli-rs/pull/447))
- Fixed cli command for memo parameter ([#446](https://github.com/near/near-cli-rs/pull/446))
- Use legacy keychain as a fallback storage when system keychain is not supported (e.g. WSL, Codespaces, Docker containers, CI) ([#439](https://github.com/near/near-cli-rs/pull/439))

### Other

- [**breaking**] updates near-* dependencies to 0.29 release ([#455](https://github.com/near/near-cli-rs/pull/455))
- Typos fix ([#451](https://github.com/near/near-cli-rs/pull/451))
- Added the ability to add an account ID to the account list without using the legacy keychain ([#449](https://github.com/near/near-cli-rs/pull/449))
- Updated the copyright year to 2025 ([#448](https://github.com/near/near-cli-rs/pull/448))

## [0.18.0](https://github.com/near/near-cli-rs/compare/v0.17.0...v0.18.0) - 2025-01-20

### Added

- Added the ability to get a public key for an account (#436)

### Other

- fixed migration. Added migration saving to be more clear (#434)
- handle new rate-limits for querying pools (#433)
- Extended the documentation for building NEAR CLI from the source code (#430)

## [0.17.0](https://github.com/near/near-cli-rs/compare/v0.16.1...v0.17.0) - 2024-12-16

### Added

- Added the ability to send a signed meta-transaction from a file (#428)
- Added the ability to send a signed transaction from a file (#412)
- Added the ability to use TEACH ME mode to create an account with a faucet service sponsor (#407)

### Fixed

- Fixed field name for JSON file with transaction information (signed-transaction-info.json) (#427)

### Other

- add CODEOWNERS (#426)
- print searched location on miss on sign-with-legacy-keychain (#422)
- [**breaking**] updates near-* dependencies to 0.28 release (#423)

## [0.16.1](https://github.com/near/near-cli-rs/compare/v0.16.0...v0.16.1) - 2024-12-06

### Added

- Added the ability to save payload for broadcast_tx_commit ([#413](https://github.com/near/near-cli-rs/pull/413))
- Get the final CLI command into the shell history with a small helper setup ([#415](https://github.com/near/near-cli-rs/pull/415))
- Trace configuration for loading wait indicators and --teach-me flag moved to library ([#417](https://github.com/near/near-cli-rs/pull/417))
- add to devtools workflow

## [0.16.0](https://github.com/near/near-cli-rs/compare/v0.15.1...v0.16.0) - 2024-11-18

### Other

- [**breaking**] updates near-* dependencies to 0.27 release ([#414](https://github.com/near/near-cli-rs/pull/414))

## [0.15.1](https://github.com/near/near-cli-rs/compare/v0.15.0...v0.15.1) - 2024-09-11

### Other

- updates near-* dependencies to 0.26 release ([#405](https://github.com/near/near-cli-rs/pull/405))

## [0.15.0](https://github.com/near/near-cli-rs/compare/v0.14.3...v0.15.0) - 2024-09-02

### Added
- UX improvement for signing with Ledger - automatically open a near-app on Ledger device before calling methods ([#403](https://github.com/near/near-cli-rs/pull/403))
- Extended --teach-me mode: async account details fetching RPC methods are also covered now ([#389](https://github.com/near/near-cli-rs/pull/389))

### Other
- Updated near-* dependencies to 0.25.0 (matching nearcore 2.1 release) ([#401](https://github.com/near/near-cli-rs/pull/401))

## [0.14.3](https://github.com/near/near-cli-rs/compare/v0.14.2...v0.14.3) - 2024-08-21

### Fixed
- Fixed the fallback legacy keychain path ([#398](https://github.com/near/near-cli-rs/pull/398))

## [0.14.2](https://github.com/near/near-cli-rs/compare/v0.14.1...v0.14.2) - 2024-08-21

### Added
- Upgraded cargo-dist to 0.21.1 and enabled Linux ARM64 build ([#397](https://github.com/near/near-cli-rs/pull/397))
- Prioritize searching an access key in the signer account subfolder (<credentials-dir>/<network>/<account-id>/*.json) in legacy keychain and then fallback to <credentials-dir>/<network>/<account-id>.json instead of the other way around ([#396](https://github.com/near/near-cli-rs/pull/396))

### Other
- Fixed the one-liner curl installation command

## [0.14.1](https://github.com/near/near-cli-rs/compare/v0.14.0...v0.14.1) - 2024-08-15

### Fixed
- Fixed native operating system keychain support that got broken with 0.14.0 release ([#392](https://github.com/near/near-cli-rs/pull/392))

## [0.14.0](https://github.com/near/near-cli-rs/compare/v0.13.0...v0.14.0) - 2024-08-13

### Fixed
- Require at least one access key to be selected in interactive mode when deleting a key ([#385](https://github.com/near/near-cli-rs/pull/385))
- Handle delegated stake errors gracefully and display a warning message instead of failing the view-account-summary command completely ([#382](https://github.com/near/near-cli-rs/pull/382))
- Entering the name of the function in interactive mode ([#379](https://github.com/near/near-cli-rs/pull/379))
- Fixed a typo in `inspect` output about missing ABI support ([#374](https://github.com/near/near-cli-rs/pull/374))

### Other
- removed media from the package ([#388](https://github.com/near/near-cli-rs/pull/388))
- updated near-* to 0.24.0, bumped up rust version ([#381](https://github.com/near/near-cli-rs/pull/381))
- Added videos to the README for installation process on Windows ([#378](https://github.com/near/near-cli-rs/pull/378))
- Cleaned up error message reporting by disabling env section of color_eyre report ([#380](https://github.com/near/near-cli-rs/pull/380))

## [0.13.0](https://github.com/near/near-cli-rs/compare/v0.12.0...v0.13.0) - 2024-07-30

### Added
- Automatically exec legacy JS CLI commands for full backward compatibility ([#366](https://github.com/near/near-cli-rs/pull/366))
- Added the ability to use the TEACH-ME mode ([#360](https://github.com/near/near-cli-rs/pull/360))
- Added a new subcommand to edit configuration parameters ([#367](https://github.com/near/near-cli-rs/pull/367))

### Fixed
- Fixed the fallback implementation of fetching active staking pools ([#369](https://github.com/near/near-cli-rs/pull/369))

### Other
- Fixed typos in user prompts and the guide ([#372](https://github.com/near/near-cli-rs/pull/372))

## [0.12.0](https://github.com/near/near-cli-rs/compare/v0.11.1...v0.12.0) - 2024-07-09

### Added
- Cover *all* commands from near-cli JS with the new near-cli-rs suggestions for full compatibility  ([#345](https://github.com/near/near-cli-rs/pull/345))
- Added the ability to select HD Path from the ledger ([#362](https://github.com/near/near-cli-rs/pull/362))
- Added loading indicators for "transaction" group commands and improved the prompt messages  ([#358](https://github.com/near/near-cli-rs/pull/358))

## [0.11.1](https://github.com/near/near-cli-rs/compare/v0.11.0...v0.11.1) - 2024-07-01

### Added
- Added loading indicators for "contract" group commands  ([#357](https://github.com/near/near-cli-rs/pull/357))
- Added loading indicators for "staking" group commands ([#356](https://github.com/near/near-cli-rs/pull/356))
- Added loading indicators for "tokens" group commands ([#355](https://github.com/near/near-cli-rs/pull/355))
- Added loading indicators for "accounts" group commands ([#352](https://github.com/near/near-cli-rs/pull/352))

### Other
- replace `ed25519-dalek` 1 -> 2 major version ([#359](https://github.com/near/near-cli-rs/pull/359))

## [0.11.0](https://github.com/near/near-cli-rs/compare/v0.10.2...v0.11.0) - 2024-06-19

### Added
- Added loading indicators to wait for the view-account-summary command ([#349](https://github.com/near/near-cli-rs/pull/349))
- Added loading indicators to wait for the create-account (sponsor-by-faucet-service) command ([#339](https://github.com/near/near-cli-rs/pull/339))

### Fixed
- Do not fail view-account-summary command if we could not retrieve access keys list ([#344](https://github.com/near/near-cli-rs/pull/344))

### Other
- [**breaking**] upgraded near-dependencies to the 0.23 version ([#350](https://github.com/near/near-cli-rs/pull/350))
- Provide instructions that help to resolve a problem with missing keychain ([#347](https://github.com/near/near-cli-rs/pull/347))

## [0.10.2](https://github.com/near/near-cli-rs/compare/v0.10.1...v0.10.2) - 2024-05-21

### Fixed
- Wrong console command for adding Function-Call key with unlimited allowance ([#342](https://github.com/near/near-cli-rs/pull/342))
- Fallback to non-auto-suggesting input of the keys to be deleted in interactive mode in offline mode or if there is a connectivity issue ([#338](https://github.com/near/near-cli-rs/pull/338))

## [0.10.1](https://github.com/near/near-cli-rs/compare/v0.10.0...v0.10.1) - 2024-05-07

### Added
- Display the transaction fee in NEAR and approximate $ after transaction is executed ([#333](https://github.com/near/near-cli-rs/pull/333))

## [0.10.0](https://github.com/near/near-cli-rs/compare/v0.9.1...v0.10.0) - 2024-05-03

### Added
- Added loading indicators to wait for staking properties to be viewed ([#328](https://github.com/near/near-cli-rs/pull/328))
- improved fetching staking pools ([#325](https://github.com/near/near-cli-rs/pull/325))
- Added loading indicators for waiting for the transaction to be signed ([#324](https://github.com/near/near-cli-rs/pull/324))

### Fixed
- Wrong console command for adding Function-Call key with any methods to account ([#329](https://github.com/near/near-cli-rs/pull/329))

### Other
- Support automatic config version migration ([#331](https://github.com/near/near-cli-rs/pull/331))
- Updated dependencies ([#332](https://github.com/near/near-cli-rs/pull/332))
- Refactored the command for adding Function-Call Access key ([#330](https://github.com/near/near-cli-rs/pull/330))

## [0.9.1](https://github.com/near/near-cli-rs/compare/v0.9.0...v0.9.1) - 2024-04-25

### Added
- Added loading indicators for waiting for responses from RPC ([#315](https://github.com/near/near-cli-rs/pull/315))

## [0.9.0](https://github.com/near/near-cli-rs/compare/v0.8.1...v0.9.0) - 2024-04-22

### Added
- Highlight the re-run command to make it more prominent ([#317](https://github.com/near/near-cli-rs/pull/317))
- Added ability to select contract function from NEAR ABI functions ([#314](https://github.com/near/near-cli-rs/pull/314))
- Added the ability to output a signed transaction (serialized as base64) to a file ([#313](https://github.com/near/near-cli-rs/pull/313))

### Other
- Updated "interactive_clap" to 0.2.10 ("flatten" parameter changed to "subargs") ([#322](https://github.com/near/near-cli-rs/pull/322))
- fix typos ([#318](https://github.com/near/near-cli-rs/pull/318))
- update `near-ledger` to `0.5.0` ([#309](https://github.com/near/near-cli-rs/pull/309))
- Upgraded `inquire` crate to use CustomType inputs where initial value is needed ([#310](https://github.com/near/near-cli-rs/pull/310))

## [0.8.1](https://github.com/near/near-cli-rs/compare/v0.8.0...v0.8.1) - 2024-02-26

### Fixed
- Added support for viewing account summary on networks without NEAR Social contract ([#302](https://github.com/near/near-cli-rs/pull/302))

### Other
- Improved formatting of the Install section in the README

## [0.8.0](https://github.com/near/near-cli-rs/compare/v0.7.8...v0.8.0) - 2024-02-19

### Added
- Added `inspect` and `download-abi` commands for contracts! ([#293](https://github.com/near/near-cli-rs/pull/293))

### Fixed
- Fixed incorrect serialization in staking delegation commands that required to input amounts ([#300](https://github.com/near/near-cli-rs/pull/300))
- Fixed a syntax error in CI (publish-to-npm.yml)

## [0.7.8](https://github.com/near/near-cli-rs/compare/v0.7.7...v0.7.8) - 2024-02-03

### Other
- Updated binary releases pipeline to use cargo-dist v0.9.0 (previously v0.7.2) ([#294](https://github.com/near/near-cli-rs/pull/294))
- Updated send-ft command ([#283](https://github.com/near/near-cli-rs/pull/283))

## [0.7.7](https://github.com/near/near-cli-rs/compare/v0.7.6...v0.7.7) - 2024-01-19

### Added
- Updated dialog for entering arguments to a function (as-read-only) ([#285](https://github.com/near/near-cli-rs/pull/285))

### Other
- Updated binary releases pipeline to use cargo-dist v0.7.2 (previously v0.3.0) ([#289](https://github.com/near/near-cli-rs/pull/289))
- Avoid unnecessary "interactive_clap::FromCli" implementations ([#288](https://github.com/near/near-cli-rs/pull/288))

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
- Exposed sponsor_by_faucet_service module to reuse in "cargo-near" ([#246](https://github.com/near/near-cli-rs/pull/246))

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
