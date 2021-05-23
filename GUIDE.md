## User Guide

This guide is intended to give an elementary description of near-cli and an
overview of its capabilities. This guide assumes that near-cli is
[installed](README.md#installation)
and that readers have passing familiarity with using command line tools. This
also assumes a Unix-like system, although most commands are probably easily
translatable to any command line shell environment.

### Actions

* [Add access key, contract code, stake proposal, sub-account, implicit-account](#add)
* [Construct a new transaction](#construct-transaction)
* [Delete access key, account](#delete)
* [Execute function (contract method)](#execute)
* [Transfer tokens](#transfer)
* [Helpers](#helpers)
* [View account, contract code, contract state, transaction](#view)


### Add access key, contract code, stake proposal, sub-account, implicit-account

* [Add a new access key for an account](#add-access-key)
* [Add a new contract code](#add-contract-code)
* [Add an implicit-account](#add-implicit-account)
* [Add a new stake proposal](#add-stake-proposal)
* [Add a new sub-account](#add-sub-account)


### Construct a new transaction

<details><summary>Construct a new transaction</summary>
<p>
</p><pre><code>To construct a transaction you will need to provide information about sender (signer) and receiver accounts, and actions that needs to be performed.

Do you want to derive some information required for transaction construction automatically querying it online?
 Yes, I keep it simple
&gt;  No, I want to work in no-network (air-gapped) environment

What is the account ID of the sender? ... frol4.testnet

What is the account ID of the receiver? ... qq.frol4.testnet

Select an action that you want to add to the transaction:
&gt; Transfer NEAR Tokens
  Call a Function
  Stake NEAR Tokens
  Create an Account
  Delete an Account
  Add an Access Key
  Detete an Access Key
  [Skip adding a new action]

How many NEAR Tokens do you want to transfer? ... 3.14 NEAR

Select an action that you want to add to the transaction:
...
&gt; [Skip adding a new action]

Would you like to sign the transaction? ...
  Yes, I want to sign the transaction with my private key
&gt; No, I want to construct the transaction and sign it somewhere else

Some extra information needs to be filled in before we proceed signing...

Enter transaction nonce (query the access key information with `near-cli utils view-access-key frol4.testnet ed25519:...` incremented by 1):

Enter recent block hash (see above or `near-cli inspect-data blocks latest`):

Constructing the Transaction...
Transaction {
...
}
Base64-encoded Transaction: ...(base64 value)...
Transaction Hash (sha256): 83ffe880d749ea081c12cb7aa96b481bea3ee0c30e13650d363c2a28edfc0971

If you want to sign the transaction using `near-cli` on another device, run: `near-cli utils sign-transaction ...(base64 value)...`.

Once you sign the Transaction Hash, we can proceed.
Enter the signature (base58-encoded string):
Signature is valid.

Constructing the Signed Transaction...
SignedTransaction {
...
}

Here is the Signed Transaction ready for submittion through RPC: ...(base64 value)...
</code></pre>
<p></p>
</details>


### Delete access key, account



### Execute function (contract method



### Transfer tokens



### Helpers



### View account, contract code, contract state, transaction


