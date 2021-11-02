login
create-account
    - implicit
    - subaccount
manage-account
    <!-- Should we add local key management? -->
    - keys
        - view
        - add
        - delete
        - nonce
    - contract
        - deploy
        - dev-deploy
        - state
        - code
        <!-- TODO: what is the best name for it? -->
        - change-function-call
        <!-- TODO: what is the best name for it? -->
        - view-function-call
    - state
    - delete
transfer
<!-- TODO: should it be manage-account? Should it be here at all? If yes, where is `unstake`? Maybe it should be an extension, like "near-cli-extension-staking".  -->
stake
<!-- TODO: feels a bit out of place, but definitely important. Maybe we need to add other view functions and have 1 subcommand for them. -->
transaction-status
helpers
    <!-- TODO: some or all of these helpers can live in extensions, let's discuss it -->
    - constract-transaction
    - recent-block-hash
    - new-stake-proposal
    - sign a transaction with the private key
	- combine unsigned transaction with a signature
	- sign a transaction with ledger
	- send signed transaction
	- deserializing the bytes from base64
	- get the public key from ledger device
    - generate key


NEAR CLI is built for:
- NEAR smart-contract developers
- validators
- ..?
Mental model to distinguish extension from the core commands:
- core commands are used by all groups of users
- extensions are used by a particular group or several groups of users
- extensions are not composable (or are they?)

Open questions:
- where to add flags like --verbose, --structured, etc.
- multiple keys management (or postpone)
- What about prompts like: `Are you sure that you want to delete the last existing key?`. Probably they should be asked after the command is entered.

Other:
- interactive mode should look like: "command - description". It will help people to learn the commands. 
