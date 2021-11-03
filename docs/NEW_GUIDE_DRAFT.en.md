`NEAR CLI Core commands`
login
create-account
    - implicit
    - subaccount
<!-- account Id is entered in this step -->
manage-account
    - keys
        - view
        - add
        - delete
    - local-keys
        <!-- Needs to be designed (with multikey management) -->
    - contract
        - deploy-code
        - dev-deploy-code
        - get-code-checksum
        - get-state
        - call-view-function
        - call-change-function
    - state
    - delete
transfer
<!-- TODO: feels a bit out of place, but definitely important. Maybe we need to add other view functions and have 1 subcommand for them. Or it can be one of the helpers-->
transaction-status
helpers
    <!-- please, suggest other helpers -->
    - generate key
	- get the public key from ledger device
    - recent-block-hash

<!-- Currently it's a part of NEAR CLI Rust. Seems like a functionality for advanced users -->
`transaction-constructor` extension
    - constract-transaction
    - sign a transaction with the private key
	- combine unsigned transaction with a signature
	- sign a transaction with ledger
	- send signed transaction
	- deserializing the bytes from base64

<!-- Most of the people will not use staking functionality from CLI, lets move it to extension. -->
`staking` extension
    <!-- TODO: what is this command? -->
    - new-stake-proposal
    - stake
    <!-- NOTE: new command, should it be here? -->
    - unstake

NEAR CLI is built for:
- NEAR dApp developers, who build smart-contracts, UIs, and tooling on NEAR
- validators
- tech-savvy people automating their routines

Mental model to distinguish extension from the core commands:
- `NEAR CLI core` is a single binary and contains all of the `core` commands
- Each extension is a separate binary that can be executed from `NEAR CLI`
- There is no extensions that are installed `by default`
- `NEAR CLI` Core commands should be usefull for all groups of users
- Extensions are used by a particular group or several groups of users
- Extensions are not composable

Open questions:
- where to add flags like --verbose, --structured, etc.
- What about prompts like: `Are you sure that you want to delete the last existing key?`. Probably they should be asked after the command is entered.

Other:
- interactive mode should look like: "command - description". It will help people to learn the commands. 
