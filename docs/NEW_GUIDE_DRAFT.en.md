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
    - ...

`Core NEAR CLI` commands:
login
create-account
    - implicit
    - subaccount
manage-account `<- accountId is entered in this step`
    - get-state
    - keys
        - view
        - add
        - delete
    - contract
        - deploy-code
        - dev-deploy-code
        - get-code-checksum
        - get-state
        - call-view-method
        - call-change-method
    - delete
transfer
helpers
    - generate-key
	- get-public-key-from-ledger

--------------------------------------------------

Extensions design is a work in progress. They are here mostly to show that we haven't forgotten about particular functionality and that this functionality will not be a part of `Core NEAR CLI`.

`developer` extension
    - ...

`explorer` extension
    - get-recent-block-hash
    - transaction-status
    - epoch-status
    - ...

`transaction-constructor` extension
    - constract-transaction
    - sign-transaction-with-private-key
	- combine-unsigned-transaction-with-signature
	- sign-transaction-with-ledger
	- send-signed-transaction
	- deserialize-bytes-from-base64
    - ...

`staking-for-delegators` extension
    - new-stake-proposal
    - stake
    - unstake
    - ...

`validators` extension
    - validators
    - proposals
    - ...

Open questions:
- Where to add flags like `--verbose`, `--structured/json/csv`, etc.
- We should add `local-keys` management option. Needs to be designed. Can live in `manage-account` -> `local-keys` or at the top level of the `Core NEAR CLI`.
- Should we have helpers in `Core NEAR CLI`? Is't that contradicts extensions concept?

Other:
- Interactive mode should look like: "command - description". It will help people to learn the commands.