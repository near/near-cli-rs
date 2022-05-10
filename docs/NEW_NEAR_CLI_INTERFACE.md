# New NEAR CLI interface

`NEAR CLI` is using `extensions` to saticfy all groups of users.
- `Core NEAR CLI` commands should be usefull for all groups of users.
- `Extensions` are used by a particular group or several groups of users.
- `Core NEAR CLI` is a single binary, there is no extensions that are installed `by default`.
- Each extension is a separate binary that can be installed and executed from `NEAR CLI`.
- `Core NEAR CLI` extensions system only allows to introduce top-level commands through its design.

`NEAR CLI` is built for:
- NEAR `dApp developers`, who build smart-contracts, UIs, and tooling on NEAR.
- `Tech-savvy` people automating their routines.
- `Validators`

`NEAR CLI UX principles`
- All altering actions should have a confirmation step with an option to skip confirmation with an explicit command line parameter (e.g. `send` at the end of the command)
- All direct children commands of a single parent command should be aligned (either represent an action or a resource, but never a mix of those on the same hierarchy level): `contract` -> `state` (resource) and `contract` -> `deploy` (action) are not aligned, so it should be either `contract` -> `get-state` + `contract` -> `deploy` or `contract` -> `state` -> `view` + `contract` -> `code` -> `deploy`
- Interactive mode should look like: `command - description`. It will help people to learn the commands.

## `Core NEAR CLI` commands

```
account
  - view-account <account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
 
  - create-sub-account <new-account-id> <initial-balance> 
    (we will treat everything after the first dot in account id as the parent account (transaction signer))
    - with-plaintext-public-key "ed25519:..." network <"mainnet"|"testnet"|...>
      - transaction signature options here (see below)
    - with-generated-keypair network <"mainnet"|"testnet"|...>
      - transaction signature options here (see below)

  - delete-account <account-id> beneficiary <beneficiary-account-id> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - list-keys <account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - add-key <account-id>
    - use-plaintext-public-key "ed25519:..." network <"mainnet"|"testnet"|...>
      - transaction signature options here (see below)
    - generate-keypair network <"mainnet"|"testnet"|...>
      - transaction signature options here (see below)

  - delete-key <account-id> <public-key> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)

  - propose-stake <account-id> <stake-amount-in-NEAR> <validation-node-public-key>
    (maybe we should extract it into the validators extension)
    - transaction signature options here (see below)

  - TODO: "import account" (aka "login") commands
```

```
contract
  - call-view-function <account-id> <function-name> <function-args> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - call-change-function <account-id> <function-name> <function-args> --prepaid-gas <prepaid-gas> --attached-deposit <deposit-amount> network <"mainnet"|"testnet"|...>
    - transaction signature options here (see below)
  
  - deploy <account-id> use-file <path-to-wasm-file>

    - with-init-call <function-name> <function-args> --prepaid-gas <prepaid-gas> --attached-deposit <deposit-amount>
      - transaction signature options here (see below)
    - no-init-call
      - transaction signature options here (see below)

  - download <account-id> to-folder <path-to-download-folder> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>

  - view-state <account-id> key-prefix <storage-key-prefix> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
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
  
  - view-nft-list <nft-contract-account-id> network <"mainnet"|"testnet"|...> <now|at-timestamp|at-block-height|at-block-hash>
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
  * `sign-with-keychain`
  * `sign-with-ledger`
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
- validators
- proposals
- ...
```

### `Other` extensions
- `NFT`
- `FT`
