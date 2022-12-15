# New NEAR CLI interface

NEAR CLI is built for:
- **NEAR dApp developers**, who build smart-contracts, UIs, and tooling on NEAR.
- **Tech-savvy people** automating their routines.
- **Validators**

NEAR CLI is using extensions to satisfy all groups of users.
- **Core NEAR CLI** commands should be usefull for all groups of users.
- **Extensions** are used by a particular group or several groups of users.
- **Core NEAR CLI** is a single binary, there is no extensions that are installed *by default*.
- Each extension is a separate binary that can be installed and executed from **NEAR CLI**.
- **Core NEAR CLI** extensions system only allows to introduce top-level commands through its design.


NEAR CLI UX principles:
- All altering actions should have a confirmation step with an option to skip confirmation with an explicit command line parameter (e.g. `send` at the end of the command)
- All direct children commands of a single parent command should be aligned (either represent an action or a resource, but never a mix of those on the same hierarchy level): `contract` -> `state` (resource) and `contract` -> `deploy` (action) are not aligned, so it should be either `contract` -> `get-state` + `contract` -> `deploy` or `contract` -> `state` -> `view` + `contract` -> `code` -> `deploy`
- Interactive mode should look like: `command - description`. It will help people to learn the commands.

## Core NEAR CLI commands

```
account
  - view-account-summary <account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - create-account
    How do you cover the costs of account creation?
    - sponsor-by-linkdrop (mainnet)
    - sponsor-by-... (mainnet)
    - sponsor-by-wallet (testnet only)
    - fund-myself <new-account-id> <initial-balance>
      - autogenerate-new-keypair
        - save-to-keychain network <"mainnet"|"testnet"|...>
          - transaction signature options here (see below)
        - print-to-terminal network <"mainnet"|"testnet"|...>
          - transaction signature options here (see below)
      - use-manually-provided-seed-prase "twelve words goes here" network <"mainnet"|"testnet"|...>
        - transaction signature options here (see below)
      - use-manually-provided-public-key "ed25519:..." network <"mainnet"|"testnet"|...>
        - transaction signature options here (see below)
    - fund-later <initial-balance>  (implicit account creation)

  - import-account (a.k.a "log in" / "sign in")
    - using-web-wallet network-config <"mainnet"|"testnet">
    - using-seed-phrase <seed-phrase> --hd-path "m/44'/397'/0'" network-config <"mainnet"|"testnet">
    - using-private-key <private-key> network-config <"mainnet"|"testnet">

  - delete-account <account-id> beneficiary <beneficiary-account-id> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - list-keys <account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - add-key <account-id>
    - grant-full-access
      - autogenerate-new-keypair
        - save-to-keychain network <"mainnet"|"testnet"|...>
          - transaction signature options here (see below)
        - print-to-terminal network <"mainnet"|"testnet"|...>
          - transaction signature options here (see below)
      - use-manually-provided-seed-phrase "twelve words goes here" network <"mainnet"|"testnet"|...>
        - transaction signature options here (see below)
      - use-manually-provided-public-key "ed25519:..." network <"mainnet"|"testnet"|...>
        - transaction signature options here (see below)
    - grant-function-call-access --receiver-account-id <account-id> --method-names 'comma,separated,list' --allowance '0.25NEAR'
      - (use the same follow-up parameters as for `grant-full-access`)

  - delete-key <account-id> <public-key> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

```

```
contract
  - call-function
    - as-read-only <account-id> <function-name> <function-args> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
    - as-transaction <account-id> <function-name> <function-args> --prepaid-gas <prepaid-gas> --attached-deposit <deposit-amount> network <"mainnet"|"testnet"|...>
      - transaction signature options here (see below)

  - deploy <account-id> use-file <path-to-wasm-file>

    - with-init-call <function-name> <function-args> --prepaid-gas <prepaid-gas> --attached-deposit <deposit-amount>
      - transaction signature options here (see below)
    - without-init-call
      - transaction signature options here (see below)

  - download-wasm <account-id> to-folder <path-to-download-folder> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
```

```
tokens <owner-account-id>
  - send-near <receiver-account-id> <amount-in-NEAR> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - send-ft <ft-contract-account-id> <receiver-account-id> <amount-in-fungible-tokens> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - send-nft <nft-contract-account-id> <receiver-account-id> <token-id> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - view-near-balance network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - view-ft-balance <ft-contract-account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - view-nft-assets <nft-contract-account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
```

```
transaction
  - view-status <transaction-hash> <signer-account-id> network <"mainnet"|"testnet"|...>

  - construct-transaction (TODO: keep the current command structure for now)
```

```
extensions
(WIP)
  - explore
  - install
  - list-installed
  - uninstall
```

```
config
(WIP)
  - connections
    - show-selected
    - select <connection-name>
    - list
    - add <connection-name> <> <network-name>  <url1> <url1> <...>
    - delete <connection-name>
  - cli
    - set <parameter> <value>
    - get <parameter>
```

```
local-keys
(WIP: maybe merge into the `account` command)
  - add-using
    - near-wallet
    - seed-phrase <seed-phrase>
    - ledger
    - private-key <private-key>
  - generate
```

Transaction signature options:
  * `sign-with-macos-keychain`
  * `sign-with-keychain`
  * `sign-with-ledger`
  * `sign-with-access-key-file <path.json>`
  * `sign-with-seed-phrase <seed-phrase> --hd-path "m/44'/397'/0'"`
  * `sign-with-plaintext-private-key "ed25519:..."`

### Top-level `Core NEAR CLI` flags
```txt
--verbose (print all available error info)
--json (show answer in json format)
--scripting (turn off interactive mode)
```

## Extensions
Extensions design is a work in progress. They are here mostly to show that we haven't forgotten about particular functionality and that this functionality will not be a part of `Core NEAR CLI`.

### `developer` extension
```txt
- dev-deploy-code <wasm-file>
- inspect-storage <account-id> key-prefix <storage-key-prefix> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
- ...
```

### `explorer` extension
```txt
- get-recent-block-hash
- get-transaction-status <transaction-hash>
- get-epoch-status <epoch>
- ...
```

### `transaction-constructor` extension
```txt
- constract-transaction
- sign-transaction-with-private-key
- combine-unsigned-transaction-with-signature
- sign-transaction-with-ledger
- send-signed-transaction
- deserialize-bytes-from-base64
- ...
```

### `staking-for-delegators` extension
```txt
- make-new-stake-proposal
- stake
- unstake
- ...
```

### `validators` extension
```txt
- stake
- validators
- proposals
- ...
```

### Other extensions
- `NFT`
- `FT`
- `lockup`
- `multisig`
