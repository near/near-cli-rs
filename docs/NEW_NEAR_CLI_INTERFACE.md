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
```txt
account
    - create
        - implicit
        - subaccount <master-account> <new-subaccount-id>
    - manage <account-id>
        - state
            - view
            - transfer-tokens <sender> <reciever> <amount>
        - keys
            - view
            - add
            - delete <public-key>
        - contract
            - code
                - deploy <wasm-path>
                - view-code-checksum
            - state
                - view
            - call
                - view-function
                - change-function
    - delete

local-keys
    - add-using
        - near-wallet
        - seed-phrase <seed-phrase>
        - ledger
        - private-key <private-key>
    - generate

mange-config
    - connections
        - show-selected
        - select <connection-name>
        - list
        - add <connection-name> <> <network-name>  <url1> <url1> <...>
        - delete <connection-name>
    - cli
        - set <parameter> <value>
        - get <parameter>

extensions
    - explore
    - install
    - list-installed
    - uninstall
```

### To level `Core NEAR CLI` flags
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