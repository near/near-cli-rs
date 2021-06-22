## User Guide

This guide is intended to give a detailed description of _near-cli_ and an
overview of its capabilities. This guide assumes that _near-cli_ is
[installed](README.md#installation)
and that readers have passing familiarity with using command line tools. This
also assumes a Unix-like system, although most commands are probably easily
translatable to any command line shell environment.

Before proceeding to the description of specific commands, it is necessary to consider two points common to these commands:

1. _Online_ / _Offline_ mode

    The result of the creation of any transaction in the _Offline_ mode (display)  is a prepared transaction in the form of a Base64-encoded string, which can then be [sent to the network via RPC](https://docs.near.org/docs/api/rpc#transactions). In _Online_ mode, such a function has already been added (send).
    Also, the _Online_ mode provides automatic collection of data necessary to form and send a transaction, and in the _Offline_ mode, you will need to enter some parameters manually:

      * _transaction nonce_

        This number will need to be obtained using [View a nonce](#view-a-nonce), increasing by 1

      * _recent block hash_

        Information for this parameter should be found [here](https://explorer.testnet.near.org/blocks).

    <details><summary><i>recent block hash</i></summary>
        <img src="media/blocks.png" width="836"/>
        <img src="media/block_hash.png" width="836"/>
    </details>

    <details><summary><i>Demonstration of the _Offline_ mode</i></summary>
    <a href="https://asciinema.org/a/REcIXg1yQqLpz42EzNQt8B99f?autoplay=1&t=1&speed=2">
        <img src="https://asciinema.org/a/REcIXg1yQqLpz42EzNQt8B99f.png" width="836"/>
    </a>
    </details>

2. Sign transaction

    _near-cli_ offers several ways to sign the created transaction. Let's take a closer look at each.

      * _I want to sign the transaction with my private key_

        When choosing this signature option, _near-cli_ will ask the user to enter access keys:
          "public_key":"ed25519:Ebx7...",
          "private_key":"ed25519:2qM8..."

      * _I want to sign the transaction with keychain_
        
        If you select this signature option, _near-cli_ will find the access keys itself in the *username.json* file located in */Users/user/.near-credentials/default/*.  
        For example, */Users/frovolod/.near-credentials/default/volodymyr.testnet.json*.

      * _I want to construct the transaction and sign it somewhere else_
        
        This option assumes that a third-party software product will sign the created transaction.


### Actions

* [View account, contract code, contract state, transaction, nonce](#view-account-contract-code-contract-state-transaction-nonce)
* [Transfer tokens](#transfer-tokens)
* [Execute function (contract method)](#execute-function-contract-method)
* [Add access key, contract code, stake proposal, sub-account, implicit-account](#add-access-key-contract-code-stake-proposal-sub-account-implicit-account)
* [Delete access key, account](#delete-access-key-account)
* [Construct a new transaction](#construct-a-new-transaction)
* [Helpers](#helpers)


### View account, contract code, contract state, transaction, nonce

It is possible to [View properties for an account](#view-properties-for-an-account), to [View a contract code](#view-a-contract-code) and to [View a contract state](#view-a-contract-state) in real time (__at-final-block__) and in the mode of archived data (__at-block-height__, __at-block-hash__). The examples below show how these modes can be used.

#### View properties for an account

##### at-final-block

To view account information, in the terminal command line type:
```txt
./near-cli view account-summary \
        network testnet \
        account 'volodymyr.testnet' \
        at-final-block
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'volodymyr.testnet' at block #49787790 (33RYuu9YoLDVCidWig8uKtuSb7jr3NmcJDzVeeKmSkiW)
Native account balance: 256.718 NEAR
Validator stake: 0 NEAR
Storage used by the account: 115385 bytes
Contract code SHA-256 checksum (hex): 4de7df8ee6ff3780cfed298ceafde26e7477041ca8e2af7ae8c749de7068c0f2
Number of access keys: 7
   1. ed25519:2KZwhWEM5hbtP28kpx9TER3zyz9rL3Az1fcHsgr2Fzd1 (nonce: 9) is granted to full access
   2. ed25519:36u45LFDfgKQYr8ApgBi1kUHN5FpkTQRqvt66cwEdqiK (nonce: 0) is granted to full access
   3. ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS (nonce: 168) is granted to full access
   4. ed25519:8b6ghV9BLNNB7RwaxSxjbH636bfM9m6NL7bmLBSXPqK8 (nonce: 2) is granted to full access
   5. ed25519:937VyVwszEH13quNuPM4nNQTrDssNzRNnLMjPbN92tgu (nonce: 3) is granted to full access
   6. ed25519:ApWodksvd7grTjFFimCroLxvrVDi7WD6g6gDE7RuhoGH (nonce: 1) is granted to full access
   7. ed25519:JC5R9H6wjthHeumnUePRjvJNJrRm6ZTFUoi1NYuj9DBZ (nonce: 0) is granted to full access
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/8sdSrqPbi1TVaoZgQ5luKJgod?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/8sdSrqPbi1TVaoZgQ5luKJgod.png" width="836"/>
</a>
</details>

##### at-block-height

To view account information, in the terminal command line type:
```txt
./near-cli view account-summary \
        network testnet \
        account 'volodymyr.testnet' \
        at-block-height 42775277
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'volodymyr.testnet' at block #42775277 (FrMNHB5y2fZZVfgC6VgfykSRWoTViT2KeWGhgkXKgtTe)
Native account balance: 377.874 NEAR
Validator stake: 0 NEAR
Storage used by the account: 592 bytes
Contract code is not deployed to this account.
Number of access keys: 6
   1. ed25519:36u45LFDfgKQYr8ApgBi1kUHN5FpkTQRqvt66cwEdqiK (nonce: 0) is granted to full access
   2. ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS (nonce: 111) is granted to full access
   3. ed25519:8b6ghV9BLNNB7RwaxSxjbH636bfM9m6NL7bmLBSXPqK8 (nonce: 1) is granted to full access
   4. ed25519:937VyVwszEH13quNuPM4nNQTrDssNzRNnLMjPbN92tgu (nonce: 3) is granted to full access
   5. ed25519:ApWodksvd7grTjFFimCroLxvrVDi7WD6g6gDE7RuhoGH (nonce: 1) is granted to full access
   6. ed25519:JC5R9H6wjthHeumnUePRjvJNJrRm6ZTFUoi1NYuj9DBZ (nonce: 0) is granted to full access
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
</details>

##### at-block-hash

To view account information, in the terminal command line type:
```txt
./near-cli view account-summary \
        network testnet \
        account 'volodymyr.testnet' \
        at-block-hash BWUZMBHPnsQR1u69keUZcYKpsjREqtGFNNGkXq74c8JN
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'volodymyr.testnet' at block #42466273 (BWUZMBHPnsQR1u69keUZcYKpsjREqtGFNNGkXq74c8JN)
Native account balance: 377.874 NEAR
Validator stake: 0 NEAR
Storage used by the account: 592 bytes
Contract code is not deployed to this account.
Number of access keys: 6
   1. ed25519:36u45LFDfgKQYr8ApgBi1kUHN5FpkTQRqvt66cwEdqiK (nonce: 0) is granted to full access
   2. ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS (nonce: 111) is granted to full access
   3. ed25519:8b6ghV9BLNNB7RwaxSxjbH636bfM9m6NL7bmLBSXPqK8 (nonce: 1) is granted to full access
   4. ed25519:937VyVwszEH13quNuPM4nNQTrDssNzRNnLMjPbN92tgu (nonce: 3) is granted to full access
   5. ed25519:ApWodksvd7grTjFFimCroLxvrVDi7WD6g6gDE7RuhoGH (nonce: 1) is granted to full access
   6. ed25519:JC5R9H6wjthHeumnUePRjvJNJrRm6ZTFUoi1NYuj9DBZ (nonce: 0) is granted to full access
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
</details>

#### View a contract code

Real-time (__at-final-block__) and archived data (__at-block-height__, __at-block-hash__) modes should be used in the same way as [View properties for an account](#view-properties-for-an-account).  

  * In order to get the contract file, type the following in the terminal command line:

    ```txt
    ./near-cli view contract-code \
            network testnet \
            contract 'volodymyr.testnet' \
            download './contract-volodymyr.testnet.wasm' \
            at-final-block
    ```

    <details><summary><i>The result of this command will be as follows:</i></summary>
    
    ```txt
    The file "contract-volodymyr.testnet.wasm" was downloaded successfully
    ```
    </details>

    <details><summary><i>Demonstration of the command in interactive mode</i></summary>
    <a href="https://asciinema.org/a/4sJOszIaaH3zfGzpWAlR9Gxpk?autoplay=1&t=1&speed=2">
        <img src="https://asciinema.org/a/4sJOszIaaH3zfGzpWAlR9Gxpk.png" width="836"/>
    </a>
    </details>

  * In order to get the contract hash, type the following in the terminal command line:

    ```txt
    ./near-cli view contract-code \
            network testnet \
            contract 'volodymyr.testnet' \
            hash \
            at-final-block
    ```
    
    <details><summary><i>The result of this command will be as follows:</i></summary>

    ```txt
    Hash of the contract: 6F7TqH3ggf4iKbmJpFg9uJq87HTUohP1rdrDU7tZx9iH
    ```
    </details>

    <details><summary><i>Demonstration of the command in interactive mode</i></summary>
    <a href="https://asciinema.org/a/JQPVxVliUflaVqUeyfOShXqqQ?autoplay=1&t=1&speed=2">
        <img src="https://asciinema.org/a/JQPVxVliUflaVqUeyfOShXqqQ.png" width="836"/>
    </a>
    </details>

#### View a contract state

Real-time (__at-final-block__) and archived data (__at-block-height__, __at-block-hash__) modes should be used in the same way as [View properties for an account](#view-properties-for-an-account).  

To view the status of the contract, type the following in the terminal command line:
```txt
./near-cli view contract-state \
        network testnet \
        account 'volodymyr.testnet' \
        at-final-block
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Contract state (values):
[
    StateItem {
        key: "U1RBVEU=",
        value: "BA==",
        proof: [],
    },
]


Contract state (proof):
[]
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/P1jqzFGpF9qjm5XbKA1yr8RqZ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/P1jqzFGpF9qjm5XbKA1yr8RqZ.png" width="836"/>
</a>
</details>

#### View a transaction status

To view the status of the desired transaction, type its hash in the terminal command line:
```txt
./near-cli view transaction \
        network testnet \
        transaction-hash 'GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank' \
        signer 'volodymyr.testnet'
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Specify the account that signed the transaction: volodymyr.testnet
Transactiion status: FinalExecutionOutcome {
    status: SuccessValue(``),
    transaction: SignedTransactionView {
        signer_id: "volodymyr.testnet",
        public_key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS,
        nonce: 165,
        receiver_id: "qweqweqwe.volodymyr.testnet",
        actions: [
            CreateAccount,
            Transfer {
                deposit: 100000000000000000000000000,
            },
            AddKey {
                public_key: ed25519:AgVv8qjZ7yix3pTo7BimT1zoDYUSTGcg73RBssC5JMRf,
                access_key: AccessKeyView {
                    nonce: 0,
                    permission: FullAccess,
                },
            },
        ],
        signature: ed25519:266jBRjvnaxe4mDyHRGwv3TJesvgRo2umJBqkZU26fRwmhVHciu3tBSLqRZFjEuqLTiwDTrFvfxpJ8Sbd2PqHHhv,
        hash: `GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank`,
    },
    transaction_outcome: ExecutionOutcomeWithIdView {
        proof: [],
        block_hash: `AQH6jDqqxpBYj5NSZv3Skg5hUZQRsn16jvDuphCTugSQ`,
        id: `GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank`,
        outcome: ExecutionOutcomeView {
            logs: [],
            receipt_ids: [
                `5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh`,
            ],
            gas_burnt: 424555062500,
            tokens_burnt: 42455506250000000000,
            executor_id: "volodymyr.testnet",
            status: SuccessReceiptId(5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh),
        },
    },
    receipts_outcome: [
        ExecutionOutcomeWithIdView {
            proof: [],
            block_hash: `DBUpiLVVDBQwSAPU8ZTE8KQnX5skDD1dTsBjJQ8kV24R`,
            id: `5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh`,
            outcome: ExecutionOutcomeView {
                logs: [],
                receipt_ids: [
                    `851GMnZZ5FJ2aDSHM34N99yVb1ZkwY8n7F8rUcvuRpUU`,
                ],
                gas_burnt: 424555062500,
                tokens_burnt: 42455506250000000000,
                executor_id: "qweqweqwe.volodymyr.testnet",
                status: SuccessValue(``),
            },
        },
        ExecutionOutcomeWithIdView {
            proof: [],
            block_hash: `BSjrH3WyKnXhD17drR94YfM725Ho59us9N4msXrrgHEw`,
            id: `851GMnZZ5FJ2aDSHM34N99yVb1ZkwY8n7F8rUcvuRpUU`,
            outcome: ExecutionOutcomeView {
                logs: [],
                receipt_ids: [],
                gas_burnt: 0,
                tokens_burnt: 0,
                executor_id: "volodymyr.testnet",
                status: SuccessValue(``),
            },
        },
    ],
}
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/HYNfgJ5Gze7fFKntubz7TW6r6?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/HYNfgJ5Gze7fFKntubz7TW6r6.png" width="836"/>
</a>
</details>

#### View a nonce

To view the _nonce_ of the desired public access key, type the following in the terminal command line:
```txt
./near-cli view nonce \
        network testnet \
        account 'volodymyr.testnet' \
        public-key 'ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS'
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
current nonce: 168  for a public key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/g2WZpFu6njjxoklP4J7Ibc5Q6?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/g2WZpFu6njjxoklP4J7Ibc5Q6.png" width="836"/>
</a>
</details>


### Transfer tokens

This command is used to transfer tokens between accounts. Please note that the amount of tokens forwarded is indicated together with the dimensional unit (this is NEAR or yoctoNEAR).
In order to execute this command, in the terminal command line type:
```txt
./near-cli transfer near \
        network testnet \
        sender 'volodymyr.testnet' \
        receiver '21.volodymyr.testnet' \
        amount  '1 NEAR' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---


<volodymyr.testnet> has transferred 1.000 NEAR to <21.volodymyr.testnet> successfully.

Transaction Id 3Xw8R58BTmoRrJufpy5utej8MzNuA11Zw6pnS8vUcCNG.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/3Xw8R58BTmoRrJufpy5utej8MzNuA11Zw6pnS8vUcCNG
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/NjCuGbC6IDyz5VsrXZrgrSF17?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/NjCuGbC6IDyz5VsrXZrgrSF17.png" width="836"/>
</a>
</details>


### Execute function (contract method)

#### Change a method

In order to execute this command, in the terminal command line type:
```txt
./near-cli execute change-method \
        network testnet \
        contract 'meta.pool.testnet' \
        call 'distribute_staking' '{}' \
            --attached-deposit '0 NEAR' \
            --prepaid-gas '3 Tgas' \
        signer 'volodymyr.testnet' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---


The "distribute_staking" call to <meta.pool.testnet> on behalf of <volodymyr.testnet> succeeded.

Transaction Id CK4ZT2NfJhqfQAjVsvoTuyJs74SY4DQfgwKC79tZGEfw.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/CK4ZT2NfJhqfQAjVsvoTuyJs74SY4DQfgwKC79tZGEfw
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/wyk4o4FltmUD00tmToCSoH24l?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/wyk4o4FltmUD00tmToCSoH24l.png" width="836"/>
</a>
</details>

#### View a method

Real-time (__at-final-block__) and archived data (__at-block-height__, __at-block-hash__) modes should be used in the same way as [View properties for an account](#view-properties-for-an-account).  

In order to execute this command, in the terminal command line type:
```txt
./near-cli execute view-method \
        network mainnet \
        contract zavodil.poolv1.near \
        call 'get_accounts' '{"from_index": 0, "limit": 3}' \
        at-final-block
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
[
  {
    "account_id": "zavodil.near",
    "unstaked_balance": "8",
    "staked_balance": "11662078846731298726428263471",
    "can_withdraw": true
  },
  {
    "account_id": "gagdiez.near",
    "unstaked_balance": "4",
    "staked_balance": "2195619736137302553610087167",
    "can_withdraw": true
  },
  {
    "account_id": "5ff98e7c85755e0f77c78eaf4a8aeca24846d8b5.lockup.near",
    "unstaked_balance": "1300000000000000000000000001",
    "staked_balance": "10759346016497580677112779659",
    "can_withdraw": false
  }
]
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/0v0TBnInhHeXv3N85dRmyXEZM?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/0v0TBnInhHeXv3N85dRmyXEZM.png" width="836"/>
</a>
</details>


### Add access key, contract code, stake proposal, sub-account, implicit-account

#### Add a new access key for an account

Let's execute the command to add a new pair of access keys to the account with the following conditions:
  * the public key will be entered manually
  * keys will have full access
  * the transaction will be signed automatically (if there is a file with access keys)

In order to execute this command, in the terminal command line type:
```txt
./near-cli add access-key \
        network testnet \
        account 'volodymyr.testnet' \
        public-key 'ed25519:Ebx7...' \
        grant-full-access \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---

Added full access key = ed25519:2Xh8n53G5YCjEj6eangH6PZjf2YdfogxikMV6q6MxFRH to volodymyr.testnet.

Transaction Id Am9bmTSaaG1SoDU2m1MyXz3cbU4uY2XxghxeHGkeBX4x.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/Am9bmTSaaG1SoDU2m1MyXz3cbU4uY2XxghxeHGkeBX4x
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/bXhJzaCUNf3FHih268NJZ8Fit?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bXhJzaCUNf3FHih268NJZ8Fit.png" width="836"/>
</a>
</details>

Let's change our parameters to add access keys:
  * keys will be generated automatically
  * keys will have functional access
  * the transaction will be signed with key pair

In order to execute this command, in the terminal command line type:
```txt
./near-cli add access-key \
        network testnet \
        account '21.volodymyr.testnet' \
        generate-keypair \
        grant-function-call-access \ 
            --receiver-id 'meta.pool.testnet' \
            --allowance '10 NEAR' \
            --method-names 'set_a, set_b' \
        sign-private-key \
            --signer-public-key ed25519:Ebx7NiwqupsshnUsEZCzgm84SQqi8LZWJx7ermrr14JF \
            --signer-secret-key  ed25519:2qM8v3nF4opam1frweMmD5h4PM3H6jtxfuAE77rpEuUx2rSGj64AzDf9xPKG76bgAs5L1oecV93etPy6xKjds2YB \
        send

```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---

Added function access key = ed25519:ECPLXJyLv3emQ9g76fxUHM98hP5MkXD5mcyRe6KiqGK2 to 21.volodymyr.testnet.

Transaction Id PTab8ENhtLJfNqsUwVpiPw3sLPySwpzJuSNX9yEHhLD.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.testnet.near.org/transactions/PTab8ENhtLJfNqsUwVpiPw3sLPySwpzJuSNX9yEHhLD
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/DOOZ2yQfUUzuRUY1S3LaktHRm?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/DOOZ2yQfUUzuRUY1S3LaktHRm.png" width="836"/>
</a>
</details>

#### Add a new contract code

In order to add a new contract, in the terminal command line type:
```txt
./near-cli add contract-code \
        network testnet \
        account '21.volodymyr.testnet' \
        contract-file '/Users/frovolod/Documents/NEAR/rust-counter/contract/target/wasm32-unknown-unknown/release/rust_counter_tutorial.wasm' \
        initialize 'increment' '{}'     \
            --attached-deposit '0 NEAR' \
            --prepaid-gas '1 TGas' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

<a href="https://asciinema.org/a/DOOZ2yQfUUzuRUY1S3LaktHRm?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/DOOZ2yQfUUzuRUY1S3LaktHRm.png" width="836"/>
</a>
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/wO3YVUR8GOh5KoYjpSWxGwlQB?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/wO3YVUR8GOh5KoYjpSWxGwlQB.png" width="836"/>
</a>
</details>

#### Add an implicit-account

This command automatically generates access keys and saves them to a file.
In order to execute this command, in the terminal command line type:
```txt
./near-cli add implicit-account \
        generate-keypair
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The data for the access key is saved in a file /Users/frovolod/.near-credentials/default/cbb8d84ac58503f2a53e07830008531a8480b8dd42db045316cd89c87d2b4862.json
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/Ub5Q13ZEKfzG4I0yKd0zinFjr?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/Ub5Q13ZEKfzG4I0yKd0zinFjr.png" width="836"/>
</a>
</details>

#### Add a new stake proposal

You can place a state using this command:
```txt
./near-cli add stake-proposal \
        network testnet \
        validator 'volodymyr.testnet' \
        amount '100 NEAR' \
        transactions-signing-public-key 'ed25519:7ctWiPKSxdvPmAbwCBDT7dBCcJ5iqP14ohwVr7vf4whJ' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---

Error: Insufficient stake 100.000 NEAR.
The minimum rate must be 28626.505 NEAR.

Transaction Id 92TdRcwGXyVCgMniE1aX8kWTzBB58s9Y5i8dHAnh74Y8.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/92TdRcwGXyVCgMniE1aX8kWTzBB58s9Y5i8dHAnh74Y8
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/nMovygVG9GyKSVU0ywtk5z0RT?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/nMovygVG9GyKSVU0ywtk5z0RT.png" width="836"/>
</a>
</details>

#### Add a new sub-account

In order to create a sub-account, in the terminal command line type:
```txt
./near-cli add sub-account \
        network testnet \
        owner-account 'volodymyr.testnet' \
        sub-account '22.volodymyr.testnet' \
        sub-account-full-access \
        generate-keypair \
        deposit '1 NEAR' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---


New account <22.volodymyr.testnet> has been successfully created.

Transaction Id He1D8DoYEkazhVw5PJ4TxNgafmsqj7Yb72YQebM3a98p.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/He1D8DoYEkazhVw5PJ4TxNgafmsqj7Yb72YQebM3a98p
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/nW78yyzNhkK09sAxSFTgk0zX8?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/nW78yyzNhkK09sAxSFTgk0zX8.png" width="836"/>
</a>
</details>


### Delete access key, account

#### Delete an access key for this account

In order to remove access keys, in the terminal command line type:
```txt
./near-cli delete access-key \
        network testnet \
        account 'volodymyr.testnet' \
        public-key 'ed25519:HaUQJkxYtTXrJVLNHEtHPmvLJ1fuUczwocm53Xn5eFJy' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---


Access key <ed25519:HaUQJkxYtTXrJVLNHEtHPmvLJ1fuUczwocm53Xn5eFJy> for account <volodymyr.testnet> has been successfully deletted.

Transaction Id 6Du4cGn1go9EQdrsRzD6D2az4jGGQUW9361cLG1GVxNp.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/6Du4cGn1go9EQdrsRzD6D2az4jGGQUW9361cLG1GVxNp
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/KSZkGbwnQbG5vGtlG2fGIrawm?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KSZkGbwnQbG5vGtlG2fGIrawm.png" width="836"/>
</a>
</details>

#### Delete this account

This command is designed to delete the current account. It is important to remember that all tokens of the deleted account will be transferred to the "_beneficiary_" account.
In order to execute this command, in the terminal command line type:
```txt
./near-cli delete account \
        network testnet \
        account '21.volodymyr.testnet' \
        beneficiary 'volodymyr.testnet' \
        sign-with-keychain \
        send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
--- Transaction sent ---


--- Transaction execution: ---


Account <22.volodymyr.testnet> has been successfully deletted.

Transaction Id EhhLMtUEmLHV4bzdvSmzv9pLyUbKChGgtWKhuPRXx2w5.

To see the transaction in the transaction explorer, please open this url in your browser:
                    
https://explorer.wallet.testnet.near.org/EhhLMtUEmLHV4bzdvSmzv9pLyUbKChGgtWKhuPRXx2w5
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/XEwJ4NVkuG7mnDiADiNPQ2L7n?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/XEwJ4NVkuG7mnDiADiNPQ2L7n.png" width="836"/>
</a>
</details>

### Construct a new transaction

Let's consider an example when it is necessary to perform several actions within one transaction:
1. Create an account
2. Add access keys to the created account
3. Transfer tokens to the created account

To do this, we will use the transaction constructor:

<details><summary>Construct a new transaction (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/2HPsb14qsgkr05eesfzXAKTXb?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/2HPsb14qsgkr05eesfzXAKTXb.png" width="836"/>
</a>
</details>


### Helpers

#### Generate a key pair

This utility generates access keys at random. 
<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/vwlWvmyNnxaGOFMeHb5wkwe2t?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/vwlWvmyNnxaGOFMeHb5wkwe2t.png" width="836"/>
</a>
</details>

#### Sign a transaction with secret key

This utility allows you to sign a previously formed and unsigned transaction with your keys.
<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/sjS78AfPfwltNge2Yb9uRCxSX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/sjS78AfPfwltNge2Yb9uRCxSX.png" width="836"/>
</a>
</details>

#### Combine unsigned transaction with signature

This utility allows you to sign a previously generated and unsigned transaction with your signature.
<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/HfsutLZKnWS8w1PnY1kGIUYid?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/HfsutLZKnWS8w1PnY1kGIUYid.png" width="836"/>
</a>
</details>

#### Deserializing the bytes from base64

It might be useful to view the contents of a serialized transaction (either signed or not).
Given a base64-encoded string, we should be able to view the human-readable representation.
<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/Gtb4M13a8QW5VaVmfgBLEcq3X?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/Gtb4M13a8QW5VaVmfgBLEcq3X.png" width="836"/>
</a>
</details>

#### Send signed transaction

Given the base64 encoded string, we should be able to send it for execution.
<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/4g9yN7PFBdBSeQRxPc8ydfpWs?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/4g9yN7PFBdBSeQRxPc8ydfpWs.png" width="836"/>
</a>
</details>
