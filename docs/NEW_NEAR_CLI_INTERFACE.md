# New NEAR CLI interface

`NEAR CLI` is using `extensions` to saticfy all groups of users.
- `Core NEAR CLI` commands should be usefull for all groups of users.
- `Core NEAR CLI` is a single binary, there is no extensions that are installed `by default`.
- Each extension is a separate binary that can be executed from `NEAR CLI`.
- Extensions are used by a particular group or several groups of users.
- Extensions are not composable (you should not create extensions for extensions).

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
import-account
    - from-near-wallet
    - from-seed-phrase <seed-phrase>
    - from-ledger
    - from-private-key <private-key>

create-account
    - implicit
    - subaccount <master-account> <new-subaccount-id>

manage-account <accoundId>
    - get-state
    - manage-keys
        - view
        - add
        - delete <public-key>
    - make-transfer <sender> <reciever> <amount>
    - manage-contract
        - deploy-code <wasm-path>
        - get-code-checksum
        - get-state
        - call-view-method
        - call-change-method
    - delete

manage-off-chain-keys
    - ...
    - generate-key
    - get-public-key-from-ledger

manage-connections
    - show-current
    - set-current <connection-name>
    - list-all
    - add <connection-name> <network-name> <url1> <url1>
    - delete <connection-name>

manage-cli-config
    - set <parameter> <value>
    - get <parameter>
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

## Open questions
- Where to add flags like `--verbose`, `--structured/json/csv`, `--scripting-mode`, etc.
- How to manage multiple keys? Should we have `default` key? Should they have a name like `mario-game-key-1`?
- Should we have `extensions` -> `list`/`add`/`delete` commands? We can do it in Phase 2.

## Other
- Let's get inspiration for keys and storage data from this project: https://github.com/near-examples/near-account-utils