## User Guide

This guide is intended to give a detailed description of _near CLI_ and an
overview of its capabilities. This guide assumes that _near CLI_ is
[installed](README.md#installation)
and that readers have passing familiarity with using command line tools. This
also assumes a Unix-like system, although most commands are probably easily
translatable to any command line shell environment.

With _near CLI_ you can create, sign and send transactions in _online_ mode, which is enabled by default.
In _offline_ mode, you can create and sign a transaction. The base64 encoding transaction can be [signed](#sign-transaction---sign-previously-prepared-unsigned-transaction) or [sent](#send-signed-transaction---send-a-signed-transaction) later (even from another computer). To enter the _offline_ mode, you need to set the `--offline` flag in the command:

```txt
near --offline tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-later
```

Before proceeding to the description of specific commands, it is necessary to consider two points common to these commands:

1. Sign transaction

   _near CLI_ offers several ways to sign the created transaction. Let's take a closer look at each.

   - _sign-with-macos-keychain - Sign the transaction with a key saved in macOS keychain_

     The operating system _MacOS_ has its own application _[Keychain Access](https://support.apple.com/guide/keychain-access/welcome/mac)_, with the help of which _near CLI_ will independently find access keys and sign the created transaction.

   - _sign-with-keychain - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)_

     _near CLI_ will independently find access keys and sign the created transaction.  
      Directory with access keys defined in [config](#config---manage-connections-in-a-configuration-file).
     The access keys must be in the _public-key.json_ file located in _/Users/user/.near-credentials/network-name/user-name/_  
      For example, _/Users/frovolod/.near-credentials/testnet/volodymyr.testnet/ed25519_8h7kFK4quSUJRkUwo3LLiK83sraEm2jnQTECuZhWu8HC.json_

       <details><summary><i>Demonstration of the command in interactive mode</i></summary>
       <a href="https://asciinema.org/a/30jHxm9lRevRG4K1h0GWlEciV?autoplay=1&t=1&speed=2">
           <img src="https://asciinema.org/a/30jHxm9lRevRG4K1h0GWlEciV.png" width="836"/>
       </a>
       </details>

   - _sign-with-ledger - Sign the transaction with Ledger Nano device_

     This option involves signing the created transaction using a ledger.

   - _sign-with-plaintext-private-key - Sign the transaction with a plaintext private key_

     When choosing this signature option, _near CLI_ will ask the user to enter access keys:

     - "public_key":"ed25519:Ebx7...",
     - "private_key":"ed25519:2qM8..."

   - _sign-with-access-key-file - Sign the transaction using the account access key file (access-key-file.json)_

     When choosing this signature option, _near CLI_ will ask the user to enter the path to a file that contains information about account access keys.

   - _sign-with-seed-phrase - Sign the transaction using the seed phrase_

     When choosing this signature option, _near CLI_ will ask the user to enter the mnemonic phrase associated with the account.

   - _sign-later - Prepare unsigned transaction (we'll use base64 encoding to simplify copy-pasting)_

     This option involves signing the created transaction [later](#sign-transaction---sign-previously-prepared-unsigned-transaction).

2. Actions with a signed transaction

   _near CLI_ support for meta transactions as specified in [NEP-366](https://near.github.io/nearcore/architecture/how/meta-tx.html#meta-transactions). To create it, you just need to specify a _network_ that supports meta transactions. You can find out about such support in [config](#show-connections---Show-a-list-of-network-connections). The _meta_transaction_relayer_url_ field is responsible for the ability to support meta transactions. For example:

   ```txt
   meta_transaction_relayer_url = "https://near-testnet.api.pagoda.co/relay"
   ```

   A signed transaction / meta transactions can be sent for immediate execution:

   - _send - Send the transaction to the network_

   or display in base64 format to send:

   - _display - Print only the signed transaction in base64 encoding. We will use it to send it later. ([Example](#send-signed-transaction---send-a-signed-transaction): near transaction send-signed-transaction 'EQAAAHZvb...' ...)_

### Command groups

- [account - Manage accounts](#account---Manage-accounts)
- [tokens - Manage token assets such as NEAR, FT, NFT](#tokens---Manage-token-assets-such-as-NEAR-FT-NFT)
- [contract - Manage smart-contracts: deploy code, call functions](#contract---Manage-smart-contracts-deploy-code-call-functions)
- [transaction - Operate transactions](#transaction---Operate-transactions)
- [config - Manage connections in a configuration file](#config---Manage-connections-in-a-configuration-file)

### account - Manage accounts

View account details ([View properties for an account](#view-account-summary---view-properties-for-an-account)) and view account access keys ([View a list of access keys of an account](#list-keys---View-a-list-of-access-keys-of-an-account)) is possible at the current time (**_now_**) and at a certain point in the past by specifying the block (**_at-block-height_** or **_at-block-hash_**). The examples below show how these modes can be used.

- [view-account-summary](#view-account-summary---View-properties-for-an-account)
- [import-account](#import-account---import-existing-account-aka-sign-in)
- [create-account](#create-account---Create-a-new-account)
- [delete-account](#delete-account---Delete-an-account)
- [list-keys](#list-keys---View-a-list-of-access-keys-of-an-account)
- [add-key](#add-key---Add-an-access-key-to-an-account)
- [delete-key](#delete-key---Delete-an-access-key-from-an-account)
- [manage-storage-deposit](#manage-storage-deposit---Storage-management-deposit-withdrawal-balance-review)

#### view-account-summary - View properties for an account

- [now](#now---View-properties-in-the-final-block)
- [at-block-height](#at-block-height---View-properties-in-a-height-selected-block)
- [at-block-hash](#at-block-hash---View-properties-in-a-hash-selected-block)

##### now - View properties in the final block

To view an account summary for the last block, in the terminal command line type:

```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'fro_volod.testnet' at block #97804915 (5G8HHWMJMHRMMaHTjeZLSvL7ruYMtH9tXq25Q6BPUivu)
Native account balance: 182.685021399504861699999997 NEAR
Validator stake: 0 NEAR
Storage used by the account: 288962 bytes
Contract code SHA-256 checksum (hex): fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec
Number of access keys: 14
   1. ed25519:2QFAeUutKUDpmgKDyHXm7Wcz1uhjxk92fK6zY2dB7FCD (nonce: 97492076000000) is granted to only do [] function calls on v2.ref-farming.testnet with an allowance of 0.25 NEAR
   2. ed25519:3p1HbrTDYxY4q3V6QznW14qkuv3Bq1phFpCTsbrJpbEC (nonce: 94363284000000) is granted to full access
   3. ed25519:5UJE4PzyxECS42hBZSD1QQCLdq5j39vCtzshFPbnGdm1 (nonce: 73069087000002) is granted to full access
   4. ed25519:6YU78BezKwQNrz5vmtkSCALtx7cPDC1JBs9DhjeSJ39X (nonce: 97490513000000) is granted to only do [] function calls on v2.ref-farming.testnet with an allowance of 0.25 NEAR
   5. ed25519:7YCfA1KrToJtAYGTBgAMe4LWfQEi4iwLGcH2q5SvGKzD (nonce: 94982716000000) is granted to only do [] function calls on mintspace2.testnet with an allowance of 0.25 NEAR
   6. ed25519:95w5YFsJ3iktzDwRBWUGqLF6Gv5CoJuVifBjcEEdJs8s (nonce: 72253433000003) is granted to full access
   7. ed25519:9nyDySTNAGPywxC9pG4DPdnF3eEVexDgrfzZYsoahPsV (nonce: 76057805000000) is granted to full access
   8. ed25519:AEC4szaeNzT8PQAifsnisdivq4mwswJbBM65DdkT6kdS (nonce: 72263674000000) is granted to full access
   9. ed25519:D31un5TFeABdNUVMaf3QzeBz3Z3yau2GZA2VPe8XX6GB (nonce: 72325441000021) is granted to full access
  10. ed25519:DZz4r5oLSBVcLuqFzSoLUEJ3Qv67cpgGbsRHy8SvbGiU (nonce: 72253481000000) is granted to full access
  11. ed25519:DyKmdLkWMqC1HFs6t6PfNhVemjQE16W2RNofWPpW5ZZh (nonce: 72325378000007) is granted to full access
  12. ed25519:EWoYxHNZHtApUfu1nTGC49XHW5dNinoDKABcauHnjevZ (nonce: 73069042000001) is granted to full access
  13. ed25519:EYtsL67TpgfpE1udnga2m41vDoBqeZ2DB32onhsxsVUb (nonce: 72251760000002) is granted to full access
  14. ed25519:G2U7aZ91pgG3TS96gCWov5L1DkNWSi3756RRkwuspZ4L (nonce: 72251684000002) is granted to full access
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/WA7eNU7hbmv7oa5lNLrmJzmRu?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/WA7eNU7hbmv7oa5lNLrmJzmRu.png" width="836"/>
</a>
</details>

##### at-block-height - View properties in a height-selected block

To view an account summary for a specific block, you can specify the height of that block. To do this, at the terminal command line, type:

```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    at-block-height 73069245
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'fro_volod.testnet' at block #73069245 (HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY)
Native account balance: 198.9924766125790117 NEAR
Validator stake: 0 NEAR
Storage used by the account: 288660 bytes
Contract code SHA-256 checksum (hex): fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec
Number of access keys: 12
   1. ed25519:5UJE4PzyxECS42hBZSD1QQCLdq5j39vCtzshFPbnGdm1 (nonce: 73069087000001) is granted to full access
   2. ed25519:95w5YFsJ3iktzDwRBWUGqLF6Gv5CoJuVifBjcEEdJs8s (nonce: 72253433000003) is granted to full access
   3. ed25519:AEC4szaeNzT8PQAifsnisdivq4mwswJbBM65DdkT6kdS (nonce: 72263674000000) is granted to full access
   4. ed25519:D31un5TFeABdNUVMaf3QzeBz3Z3yau2GZA2VPe8XX6GB (nonce: 72325441000009) is granted to full access
   5. ed25519:DZz4r5oLSBVcLuqFzSoLUEJ3Qv67cpgGbsRHy8SvbGiU (nonce: 72253481000000) is granted to full access
   6. ed25519:DyKmdLkWMqC1HFs6t6PfNhVemjQE16W2RNofWPpW5ZZh (nonce: 72325378000001) is granted to full access
   7. ed25519:EWoYxHNZHtApUfu1nTGC49XHW5dNinoDKABcauHnjevZ (nonce: 73069042000001) is granted to full access
   8. ed25519:EYtsL67TpgfpE1udnga2m41vDoBqeZ2DB32onhsxsVUb (nonce: 72251760000002) is granted to full access
   9. ed25519:G2U7aZ91pgG3TS96gCWov5L1DkNWSi3756RRkwuspZ4L (nonce: 72251684000002) is granted to full access
  10. ed25519:H5A5WfckocSLeXC7h22PcnscrWWrADHaRzrVWFMYT5o9 (nonce: 72254265000000) is granted to full access
  11. ed25519:HXHM2GTqDzCZnd7UQzPtL7VwcFfcm7n8Z8voo1ArE4Tr (nonce: 72263503000002) is granted to full access
  12. ed25519:HjzSeCGdWT15iSj2TybmKV2dZteu1VYYAaYvNYVNZY2W (nonce: 72253750000000) is granted to full access
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/oKL2H2gbDntOt0MHqpjsPnZZv?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/oKL2H2gbDntOt0MHqpjsPnZZv.png" width="836"/>
</a>
</details>

##### at-block-hash - View properties in a hash-selected block

To view an account summary for a specific block, you can specify the hash of that block. To do this, at the terminal command line, type:

```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    at-block-hash HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Account details for 'fro_volod.testnet' at block #73069245 (HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY)
Native account balance: 198.9924766125790117 NEAR
Validator stake: 0 NEAR
Storage used by the account: 288660 bytes
Contract code SHA-256 checksum (hex): fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec
Number of access keys: 12
   1. ed25519:5UJE4PzyxECS42hBZSD1QQCLdq5j39vCtzshFPbnGdm1 (nonce: 73069087000001) is granted to full access
   2. ed25519:95w5YFsJ3iktzDwRBWUGqLF6Gv5CoJuVifBjcEEdJs8s (nonce: 72253433000003) is granted to full access
   3. ed25519:AEC4szaeNzT8PQAifsnisdivq4mwswJbBM65DdkT6kdS (nonce: 72263674000000) is granted to full access
   4. ed25519:D31un5TFeABdNUVMaf3QzeBz3Z3yau2GZA2VPe8XX6GB (nonce: 72325441000009) is granted to full access
   5. ed25519:DZz4r5oLSBVcLuqFzSoLUEJ3Qv67cpgGbsRHy8SvbGiU (nonce: 72253481000000) is granted to full access
   6. ed25519:DyKmdLkWMqC1HFs6t6PfNhVemjQE16W2RNofWPpW5ZZh (nonce: 72325378000001) is granted to full access
   7. ed25519:EWoYxHNZHtApUfu1nTGC49XHW5dNinoDKABcauHnjevZ (nonce: 73069042000001) is granted to full access
   8. ed25519:EYtsL67TpgfpE1udnga2m41vDoBqeZ2DB32onhsxsVUb (nonce: 72251760000002) is granted to full access
   9. ed25519:G2U7aZ91pgG3TS96gCWov5L1DkNWSi3756RRkwuspZ4L (nonce: 72251684000002) is granted to full access
  10. ed25519:H5A5WfckocSLeXC7h22PcnscrWWrADHaRzrVWFMYT5o9 (nonce: 72254265000000) is granted to full access
  11. ed25519:HXHM2GTqDzCZnd7UQzPtL7VwcFfcm7n8Z8voo1ArE4Tr (nonce: 72263503000002) is granted to full access
  12. ed25519:HjzSeCGdWT15iSj2TybmKV2dZteu1VYYAaYvNYVNZY2W (nonce: 72253750000000) is granted to full access
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/TqhSdwjoc9PMxbLZtTWSnCRR5?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/TqhSdwjoc9PMxbLZtTWSnCRR5.png" width="836"/>
</a>
</details>

#### import-account - Import existing account (a.k.a. "sign in")

- [using-web-wallet](#using-web-wallet---Import-existing-account-using-NEAR-Wallet-aka-sign-in)
- [using-seed-phrase](#using-seed-phrase---Import-existing-account-using-a-seed-phrase)
- [using-private-key](#using-private-key---Import-existing-account-using-a-private-key)

#### using-web-wallet - Import existing account using NEAR Wallet (a.k.a. "sign in")

To authorize the user, in the terminal command line type:

```txt
near account \
    import-account \
    using-web-wallet \
    network-config testnet
```

You will be redirected to the browser for authorization.  
Default wallet url is https://app.mynearwallet.com/ (for testnet - https://testnet.mynearwallet.com/). But if you want to change to a different wallet url, you can use `--wallet-url` option:

```txt
near account \
    import-account \
    using-web-wallet \
    network-config testnet\
    --wallet-url 'https://testnet.mynearwallet.com/'
```

After successful authorization in _[My NEAR Wallet](https://app.mynearwallet.com/)_, you need to return to the terminal and enter your login.

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The data for the access key is saved in macOS Keychain
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/qEqxCxVMKjAWg92XhYCzWYhxO?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/qEqxCxVMKjAWg92XhYCzWYhxO.png" width="836"/>
</a>
</details>

#### using-seed-phrase - Import existing account using a seed phrase

To authorize the user, in the terminal command line type:

```txt
near account \
    import-account \
    using-seed-phrase 'rapid cover napkin accuse junk drill sick tooth poem patch evil fan' \
        --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' \
    network-config testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The data for the access key is saved in macOS Keychain
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/G9l4So0zbT3bNGekePp1tzJg5?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/G9l4So0zbT3bNGekePp1tzJg5.png" width="836"/>
</a>
</details>

#### using-private-key - Import existing account using a private key

To authorize the user, in the terminal command line type:

```txt
near account \
    import-account \
    using-private-key ed25519:5YhAaEe3G4VtiBavJMvpzPPmknfsTauzVjwK1ZjPVw2MFM6zFyUv4tSiSfCbCn78mEnMifE6iX5qbhFsWEwErcC2 \
    network-config testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The data for the access key is saved in macOS Keychain
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/KK14atSSbI8dLB3RcuyI2tfP8?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KK14atSSbI8dLB3RcuyI2tfP8.png" width="836"/>
</a>
</details>

#### export-account - Export existing account

- [using-web-wallet](#using-web-wallet---Export-existing-account-using-NEAR-Wallet-aka-sign-in)
- [using-seed-phrase](#using-seed-phrase---Export-existing-account-using-a-seed-phrase)
- [using-private-key](#using-private-key---Export-existing-account-using-a-private-key)

#### using-web-wallet - Export existing account using NEAR Wallet

To export an existing account, enter in the terminal command line:

```txt
near account \
    export-account volodymyr.testnet \
    using-web-wallet \
    network-config testnet
```

You will be redirected to the browser for authorization.  
Default wallet url is https://app.mynearwallet.com/ (for testnet - https://testnet.mynearwallet.com/). But if you want to change to a different wallet url, you can use `--wallet-url` option:

```txt
near account \
    export-account volodymyr.testnet \
    using-web-wallet \
    network-config testnet\
    --wallet-url 'https://testnet.mynearwallet.com/'
```

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/QqVhhVaBP4MP7XFDeb6arIB3S?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/QqVhhVaBP4MP7XFDeb6arIB3S.png" width="836"/>
</a>
</details>

#### using-seed-phrase - Export existing account using a seed phrase

To export an existing account, enter in the terminal command line:

```txt
near account \
    export-account volodymyr.testnet \
    using-seed-phrase \
    network-config testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Here is the secret recovery seed phrase for account <volodymyr.testnet>: "feature army carpet ..." (HD Path: m/44'/397'/0').
```

</details>

#### using-private-key - Export existing account using a private key

To export an existing account, enter in the terminal command line:

```txt
near account \
    export-account volodymyr.testnet \
    using-private-key \
    network-config testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Here is the private key for account <volodymyr.testnet>: ed25519:4TKr1c7p...y7p8BvGdB
```

</details>

#### create-account - Create a new account

- sponsor-by-linkdrop (Not implemented yet)
- [sponsor-by-faucet-service](#sponsor-by-faucet-service---I-would-like-the-faucet-service-sponsor-to-cover-the-cost-of-creating-an-account-testnet-only-for-now)
- [fund-myself](#fund-myself---I-would-like-fund-myself-to-cover-the-cost-of-creating-an-account)
- [fund-later](#fund-later---Create-an-implicit-account)

#### sponsor-by-faucet-service - I would like the faucet service sponsor to cover the cost of creating an account (testnet only for now)

testnet has a faucet (helper service) that can sponsor account creation.  
When adding your own network in the [add-connection](#add-connection---Add-a-network-connection) configurator, you can specify your service in the _faucet_url_ field.  
Access keys to the created account can be added in several ways:

- [autogenerate-new-keypair](#autogenerate-new-keypair---Automatically-generate-a-key-pair)
- [use-manually-provided-seed-prase](#use-manually-provided-seed-prase---Use-the-provided-seed-phrase-manually)
- [use-manually-provided-public-key](#use-manually-provided-public-key---Use-the-provided-public-key-manually)
- [use-ledger](#use-ledger---Use-a-ledger)

##### autogenerate-new-keypair - Automatically generate a key pair

In order to create an account, in the terminal command line type:

```txt
near account \
    create-account sponsor-by-faucet-service test_fro.testnet \
    autogenerate-new-keypair \
    save-to-keychain \
    network-config testnet \
    create
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/test_fro.testnet/ed25519_CCwvhsp3Y3BfLbfYJQJqXJA2CaSP7CRjn1t7PyEtsjej.json
The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/test_fro.testnet.json

New account <test_fro.testnet> created successfully.
Transaction ID: FnsrXbnzH1jjTWpAo1M8cZhEN5p7jyqgRPa1aqnRzxp3
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/FnsrXbnzH1jjTWpAo1M8cZhEN5p7jyqgRPa1aqnRzxp3
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/gKThQJT5rwgxLiN4EPQ1HiNnG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/gKThQJT5rwgxLiN4EPQ1HiNnG.png" width="836"/>
</a>
</details>

##### use-manually-provided-seed-prase - Use the provided seed phrase manually

This command adds a previously known mnemonic phrase to the account.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account sponsor-by-faucet-service test_fro1.testnet \
    use-manually-provided-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
    network-config testnet \
    create
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
New account <test_fro1.testnet> created successfully.
Transaction ID: D1rRpZx5AcYWzC91Jdt69qF1iqai7knUAtvdvqNA2bv
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/D1rRpZx5AcYWzC91Jdt69qF1iqai7knUAtvdvqNA2bv
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/mYTEDj9Pxe3e6hwoTnDASuv0d?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/mYTEDj9Pxe3e6hwoTnDASuv0d.png" width="836"/>
</a>
</details>

##### use-manually-provided-public-key - Use the provided public key manually

This command adds a pre-known public access key to the account.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account sponsor-by-faucet-service test_fro2.testnet \
    use-manually-provided-public-key ed25519:HVPgAsZkZ7cwLZDqK313XJsDyqAvgBxrATcD7VacA8KE \
    network-config testnet \
    create
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
New account <test_fro2.testnet> created successfully.
Transaction ID: E7rKjJiYg1BwXa6e7xMueDS8NUNjqZSN5zDRpB5sARTi
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/E7rKjJiYg1BwXa6e7xMueDS8NUNjqZSN5zDRpB5sARTi
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/uxZ7FVsK7OQTakfrgwHhL4X7D?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/uxZ7FVsK7OQTakfrgwHhL4X7D.png" width="836"/>
</a>
</details>

##### use-ledger - Use a ledger

This command adds access keys to an account using a ledger.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account sponsor-by-faucet-service test_fro3.testnet \
    use-ledger \
    network-config testnet \
    create
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
New account <test_fro3.testnet> created successfully.
Transaction ID: BStBXVisyR5FUj3ZfCAeQ1ohfwTnx2vTbYaRPLTQ5Uek
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/BStBXVisyR5FUj3ZfCAeQ1ohfwTnx2vTbYaRPLTQ5Uek
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/b0v4IhRZRxoJ91bVcPoCfe2yl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/b0v4IhRZRxoJ91bVcPoCfe2yl.png" width="836"/>
</a>
</details>

#### fund-myself - I would like fund myself to cover the cost of creating an account

With this command, you can create both a sub account and a "short name" account.  
Access keys to the created account can be added in several ways:

- [autogenerate-new-keypair](#autogenerate-new-keypair---Automatically-generate-a-key-pair)
- [use-manually-provided-seed-prase](#use-manually-provided-seed-prase---Use-the-provided-seed-phrase-manually)
- [use-manually-provided-public-key](#use-manually-provided-public-key---Use-the-provided-public-key-manually)
- [use-ledger](#use-ledger---Use-a-ledger)

##### autogenerate-new-keypair - Automatically generate a key pair

In order to create a sub-account, in the terminal command line type:

```txt
near account \
    create-account fund-myself new.fro_volod.testnet '1 NEAR' \
    autogenerate-new-keypair \
    save-to-keychain \
    sign-as \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
New account <new.fro_volod.testnet> created successfully.
Transaction ID: DRT3EpCK9iT5APyGgfcgSoLPCLCYYKtnrVgDhGLDEZFo
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/DRT3EpCK9iT5APyGgfcgSoLPCLCYYKtnrVgDhGLDEZFo

The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/new.fro_volod.testnet/ed25519_3ngtirechhepHKrzfkdgqqtwqSMtdbSLR6N1c4ivnzu6.json
The data for the access key is saved in a file "/Users/frovolod/.near-credentials/testnet/new.fro_volod.testnet.json"
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/62q0BKhCtXV8hQ3sxDpnO1CQl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/62q0BKhCtXV8hQ3sxDpnO1CQl.png" width="836"/>
</a>
</details>

In order to create a "short name" account, in the terminal command line type:

```txt
near account \
    create-account fund-myself new7.testnet '0.1 NEAR' \
    autogenerate-new-keypair \
    save-to-keychain \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
New account <new7.testnet> created successfully.
Transaction ID: GxZRjmYxZyo6X6Mn1kfuRJhfUnxsUVCiHZAZKqrLtR27
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/GxZRjmYxZyo6X6Mn1kfuRJhfUnxsUVCiHZAZKqrLtR27

The data for the access key is saved in a file "/Users/frovolod/.near-credentials/testnet/new7.testnet/ed25519_EX1qK1S1T4WxXJFLH7qZvKxnGQtcKfEEsiA4BNxAZ6mP.json"
The file: /Users/frovolod/.near-credentials/testnet/new7.testnet.json already exists! Therefore it was not overwritten.
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/MxmfDRdKPeP0VdXUiENmV2UXr?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/MxmfDRdKPeP0VdXUiENmV2UXr.png" width="836"/>
</a>
</details>

##### use-manually-provided-seed-prase - Use the provided seed phrase manually

This command adds a previously known mnemonic phrase to the account.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account fund-myself seed.volodymyr.testnet '0.1 NEAR' \
    use-manually-provided-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
New account <seed.volodymyr.testnet> created successfully.
Transaction ID: 31iA2SsxtrRzb3fD5KtsFTZni8yUi2iZboNQih9bZuDt
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/31iA2SsxtrRzb3fD5KtsFTZni8yUi2iZboNQih9bZuDt
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/gEr7nG46C5kRp1DokYAQA28Qp?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/gEr7nG46C5kRp1DokYAQA28Qp.png" width="836"/>
</a>
</details>

##### use-manually-provided-public-key - Use the provided public key manually

This command adds a pre-known public access key to the account.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account fund-myself pk.volodymyr.testnet '0.1 NEAR' \
    use-manually-provided-public-key ed25519:6jm8hWUgwoEeGmpdEyk9zrCqtXM8kHhvg8M236ZaGusS \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
New account <pk.volodymyr.testnet> created successfully.
Transaction ID: CAVAR7jx2ofnbjxFFL2JVNbLsGNWF2q2tqMEtHxXmRLi
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/CAVAR7jx2ofnbjxFFL2JVNbLsGNWF2q2tqMEtHxXmRLi
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/R90IRnacRBO3Ni4PcpbRwm6Tt?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/R90IRnacRBO3Ni4PcpbRwm6Tt.png" width="836"/>
</a>
</details>

##### use-ledger - Use a ledger

This command adds access keys to an account using a ledger.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account fund-myself ledger1.volodymyr.testnet '0.1 NEAR' \
    use-ledger \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
New account <ledger1.volodymyr.testnet> created successfully.
Transaction ID: BKJp3QdaLtnXA8xwfqyk6JfrDsDxbxqADVyuNzQmKGNL
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/BKJp3QdaLtnXA8xwfqyk6JfrDsDxbxqADVyuNzQmKGNL
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/SN2DNObpJeqI2QrN7BNjLNdU6?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/SN2DNObpJeqI2QrN7BNjLNdU6.png" width="836"/>
</a>
</details>

#### fund-later - Create an implicit-account

- [use-auto-generation](#use-auto-generation---Use-auto-generation-to-create-an-implicit-account)
- [use-ledger](#use-ledger---Use-ledger-to-create-an-implicit-account)
- [use-seed-phrase](#use-seed-phrase---Use-seed-phrase-to-create-an-implicit-account)

##### use-auto-generation - Use auto-generation to create an implicit account

This command automatically generates access keys and saves them to a file named _implicit-account-id_.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account \
    fund-later \
    use-auto-generation \
    save-to-folder /Users/frovolod/.near-credentials/implicit
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The file "/Users/frovolod/.near-credentials/implicit/1573066d3fa7a2d56357aa5ddbc84295d94c61590390000981f5900b04e2f55f.json" was saved successfully
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/qPqMPP3tKwliWw2cu5vwCRfJi?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/qPqMPP3tKwliWw2cu5vwCRfJi.png" width="836"/>
</a>
</details>

##### use-ledger - Use ledger to create an implicit account

This command generates access keys using the ledger and saves them in a file named _implicit-account-id_.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account \
    fund-later \
    use-ledger \
    save-to-folder /Users/frovolod/.near-credentials/implicit/ledger
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The file "/Users/frovolod/.near-credentials/implicit/ledger/739c872c3057cd5d812c49345248b9fdd318c8ad33ace6cf0468109eae972c8e.json" was saved successfully
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/kL5x9MXNrlSZWS83YjVkxnsf7?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/kL5x9MXNrlSZWS83YjVkxnsf7.png" width="836"/>
</a>
</details>

##### use-seed-phrase - Use seed phrase to create an implicit account

This command generates access keys using a mnemonic phrase and saves them in a file named _implicit-account-id_.
In order to execute this command, in the terminal command line type:

```txt
near account \
    create-account \
    fund-later \
    use-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
        --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' \
    save-to-folder /Users/frovolod/.near-credentials/implicit
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
The file "/Users/frovolod/.near-credentials/implicit/eca9e1a6e0fa9a6af6d046bcffa6508f90f98e646836647ecd883d1d2b1989e5.json" was saved successfully
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/rtmvhKL9eQXqIKBkvX62oi0qx?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/rtmvhKL9eQXqIKBkvX62oi0qx.png" width="836"/>
</a>
</details>

#### delete-account - Delete an account

This command is designed to delete the current account. It is important to remember that all tokens of the deleted account will be transferred to the "_beneficiary_" account.
In order to execute this command, in the terminal command line type:

```txt
near account \
    delete-account 2.fro_volod.testnet \
    beneficiary volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
Account <2.fro_volod.testnet> has been successfully deleted.
Transaction ID: EHvB47npN8Z46qhsrw5XpKmD3n3jDn4MGiD85YSqw7cy
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/EHvB47npN8Z46qhsrw5XpKmD3n3jDn4MGiD85YSqw7cy
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/bicRQEA5bhRG6e7nKaF8ghzVm?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bicRQEA5bhRG6e7nKaF8ghzVm.png" width="836"/>
</a>
</details>

#### list-keys - View a list of access keys of an account

Viewing account access keys is possible at the current time (**_now_**) and at a certain point in the past by specifying a block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

To view the list of access keys, type the following in the terminal command line:

```txt
near account \
    list-keys fro_volod.testnet \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Number of access keys: 14
   1. ed25519:2QFAeUutKUDpmgKDyHXm7Wcz1uhjxk92fK6zY2dB7FCD (nonce: 97492076000000) is granted to only do [] function calls on v2.ref-farming.testnet with an allowance of 0.25 NEAR
   2. ed25519:3p1HbrTDYxY4q3V6QznW14qkuv3Bq1phFpCTsbrJpbEC (nonce: 94363284000000) is granted to full access
   3. ed25519:5UJE4PzyxECS42hBZSD1QQCLdq5j39vCtzshFPbnGdm1 (nonce: 73069087000002) is granted to full access
   4. ed25519:6YU78BezKwQNrz5vmtkSCALtx7cPDC1JBs9DhjeSJ39X (nonce: 97490513000000) is granted to only do [] function calls on v2.ref-farming.testnet with an allowance of 0.25 NEAR
   5. ed25519:7YCfA1KrToJtAYGTBgAMe4LWfQEi4iwLGcH2q5SvGKzD (nonce: 94982716000000) is granted to only do [] function calls on mintspace2.testnet with an allowance of 0.25 NEAR
   6. ed25519:95w5YFsJ3iktzDwRBWUGqLF6Gv5CoJuVifBjcEEdJs8s (nonce: 72253433000003) is granted to full access
   7. ed25519:9nyDySTNAGPywxC9pG4DPdnF3eEVexDgrfzZYsoahPsV (nonce: 76057805000000) is granted to full access
   8. ed25519:AEC4szaeNzT8PQAifsnisdivq4mwswJbBM65DdkT6kdS (nonce: 72263674000000) is granted to full access
   9. ed25519:D31un5TFeABdNUVMaf3QzeBz3Z3yau2GZA2VPe8XX6GB (nonce: 72325441000021) is granted to full access
  10. ed25519:DZz4r5oLSBVcLuqFzSoLUEJ3Qv67cpgGbsRHy8SvbGiU (nonce: 72253481000000) is granted to full access
  11. ed25519:DyKmdLkWMqC1HFs6t6PfNhVemjQE16W2RNofWPpW5ZZh (nonce: 72325378000007) is granted to full access
  12. ed25519:EWoYxHNZHtApUfu1nTGC49XHW5dNinoDKABcauHnjevZ (nonce: 73069042000001) is granted to full access
  13. ed25519:EYtsL67TpgfpE1udnga2m41vDoBqeZ2DB32onhsxsVUb (nonce: 72251760000002) is granted to full access
  14. ed25519:G2U7aZ91pgG3TS96gCWov5L1DkNWSi3756RRkwuspZ4L (nonce: 72251684000002) is granted to full access
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/KVfcCCyj2dEHEm4TcDkjtiW6s?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KVfcCCyj2dEHEm4TcDkjtiW6s.png" width="836"/>
</a>
</details>

#### add-key - Add an access key to an account

Let's execute the command to add a new pair of access keys to the account with the following conditions:

- the public key will be entered manually
- keys will have full access
- the transaction will be signed automatically (if there is a file with access keys)
  In order to execute this command, in the terminal command line type:

```txt
near account \
    add-key fro_volod.testnet \
    grant-full-access \
    use-manually-provided-public-key ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
Added access key = ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 to fro_volod.testnet.
Transaction ID: 2oVDKopcWphN3qrUoq7XjFMpRuCUjz6jSU327q8trAQ5
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/2oVDKopcWphN3qrUoq7XjFMpRuCUjz6jSU327q8trAQ5
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/H4BfrteW1ClAzrLcRx9m8gQAV?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/H4BfrteW1ClAzrLcRx9m8gQAV.png" width="836"/>
</a>
</details>

Let's change our parameters to add access keys:

- keys will be generated automatically
- keys will have functional access
- the transaction will be signed with key pair
  In order to execute this command, in the terminal command line type:

```txt
near account \
    add-key fro_volod.testnet \
    grant-function-call-access \
        --allowance '1 NEAR' \
        --receiver-account-id 'meta.pool.testnet' \
        --method-names 'set_a, set_b' \
    autogenerate-new-keypair \
    save-to-keychain \
    network-config testnet \
    sign-with-plaintext-private-key \
        --signer-public-key ed25519:D31un5TFeABdNUVMaf3QzeBz3Z3yau2GZA2VPe8XX6GB \
        --signer-private-key  ed25519:3UVo1GAatRz12iX3CRuKAuK3MPLDD9bPf4LXJD5DkHs13er3UeJLW7aRPAVsFQ2FjopUw6DEApEngac8FPtnnkYB \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
Added access key = ed25519:27R66L6yevyHbsk4fESZDC8QUQBwCdx6vvkk1uQmG7NY to fro_volod.testnet.
Transaction ID: DaJySrNtSUZU7KPyvfUMbh6xYi9vZeMvnj4Umo7ZzdB3
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/DaJySrNtSUZU7KPyvfUMbh6xYi9vZeMvnj4Umo7ZzdB3
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/h08oydOTq3njf6mt1FNRMHGVs?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/h08oydOTq3njf6mt1FNRMHGVs.png" width="836"/>
</a>
</details>

#### delete-key - Delete an access key from an account

In order to remove access keys, in the terminal command line type:

```txt
near account \
    delete-key fro_volod.testnet \
    ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
Access key <ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3> for account <fro_volod.testnet> has been successfully deleted.
Transaction ID: 6S7bJ76QNFypUvP7PCB1hkLM7X5GxPxP2gn4rnDHMzPz
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/6S7bJ76QNFypUvP7PCB1hkLM7X5GxPxP2gn4rnDHMzPz
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/IYaNEYcMHtmSe6zKc2L63Okph?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/IYaNEYcMHtmSe6zKc2L63Okph.png" width="836"/>
</a>
</details>

#### manage-storage-deposit - Storage management: deposit, withdrawal, balance review

- [view-balance](#view-balance---View-storage-balance-for-an-account)
- [deposit](#deposit---Make-a-storage-deposit-for-the-account)
- [withdraw](#withdraw---Withdraw-a-deposit-from-storage-for-an-account-ID)

##### view-balance - View storage balance for an account

To view the account balance on the contract on the last block, you must enter in the terminal command line:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    view-balance volodymyr.testnet \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
storage balance for <volodymyr.testnet>:
 available:        1.6 MB   (15.878059999854543210876557 NEAR [  15878059999854543210876557 yoctoNEAR])
 total:            1.6 MB   (16.238949999854543210876557 NEAR [  16238949999854543210876557 yoctoNEAR])
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/mxCOOQk8xRLvY4mIhDsrapwmG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/mxCOOQk8xRLvY4mIhDsrapwmG.png" width="836"/>
</a>
</details>

##### deposit - Make a storage deposit for the account

To add a deposit to the account balance under the contract, you must enter in the terminal command line:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    deposit volodymyr.testnet '1 NEAR' \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-macos-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
<fro_volod.testnet> has successfully added a deposit of 1 NEAR to <volodymyr.testnet> on contract <v1.social08.testnet>.
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/QXXvnhz2HasKtQdT5KPVr6d1n?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/QXXvnhz2HasKtQdT5KPVr6d1n.png" width="836"/>
</a>
</details>

##### withdraw - Withdraw a deposit from storage for an account ID

To withdraw funds from the account balance under the contract, you must enter in the terminal command line:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    withdraw '0.5 NEAR' \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
<volodymyr.testnet> has successfully withdraw 0.5 NEAR from <v1.social08.testnet>.
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/veTOTpLZZ6mKHxkn0zizpXcjx?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/veTOTpLZZ6mKHxkn0zizpXcjx.png" width="836"/>
</a>
</details>

### tokens - Manage token assets such as NEAR, FT, NFT

- [send-near](#send-near---The-transfer-is-carried-out-in-NEAR-tokens)
- [send-ft](#send-ft---The-transfer-is-carried-out-in-FT-tokens)
- [send-nft](#send-nft---The-transfer-is-carried-out-in-NFT-tokens)
- [view-near-balance](#view-near-balance---View-the-balance-of-Near-tokens)
- [view-ft-balance](#view-ft-balance---View-the-balance-of-FT-tokens)
- [view-nft-assets](#view-nft-assets---View-the-balance-of-NFT-tokens)

#### send-near - The transfer is carried out in NEAR tokens

This command is used to transfer tokens between accounts. Please note that the amount of tokens forwarded is indicated together with the dimensional unit (this is NEAR or yoctoNEAR).
In order to execute this command, in the terminal command line type:

```txt
near tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
<fro_volod.testnet> has transferred 0.1 NEAR to <volodymyr.testnet> successfully.
Transaction ID: 8BbB674VDxeg36egMzdHFsCUExpkLWAWeYqEfd9u9ZaD
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/8BbB674VDxeg36egMzdHFsCUExpkLWAWeYqEfd9u9ZaD
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/U1pNSHZw812e4BHvnFGpefVs4?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/U1pNSHZw812e4BHvnFGpefVs4.png" width="836"/>
</a>
</details>

#### send-ft - The transfer is carried out in FT tokens

This command is used to transfer FT tokens between accounts. Please note that the amount of tokens forwarded is indicated together in dimensionless units.
In order to execute this command, in the terminal command line type:

```txt
near tokens \
    fro_volod.testnet \
    send-ft usdn.testnet volodymyr.testnet 10000000000000000000 \
        --prepaid-gas 100.000TeraGas \
        --attached-deposit 1yoctoNEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
The "ft_transfer" call to <usdn.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 5a7YmANdpimiqUm6WC6n4dd91b6A9PafNNhad8HWKugN
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/5a7YmANdpimiqUm6WC6n4dd91b6A9PafNNhad8HWKugN
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/uvZGcJUpufJZdB10GsQlfXwW1?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/uvZGcJUpufJZdB10GsQlfXwW1.png" width="836"/>
</a>
</details>

#### send-nft - The transfer is carried out in NFT tokens

This command is used to transfer NFT tokens between accounts.
In order to execute this command, in the terminal command line type:

```txt
near tokens \
    fro_volod.testnet \
    send-nft paras-token-v2.testnet volodymyr.testnet 1604:4 \
        --prepaid-gas 100.000TeraGas \
        --attached-deposit 1yoctoNEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
The "nft_transfer" call to <paras-token-v2.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 9q2VbakZbj5ja6GAFXpFnbtbYHijEHyT7Ry34GQ6cvLB
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/9q2VbakZbj5ja6GAFXpFnbtbYHijEHyT7Ry34GQ6cvLB
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/iFXW6ryGQSdsWML0c3qAw3qGY?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/iFXW6ryGQSdsWML0c3qAw3qGY.png" width="836"/>
</a>
</details>

#### view-near-balance - View the balance of Near tokens

Viewing the account balance is possible at the current time (**_now_**) and at a certain moment in the past by specifying the block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

To view the amount in NEAR tokens on the account, type the following in the terminal command line:

```txt
near tokens \
    fro_volod.testnet \
    view-near-balance \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
fro_volod.testnet account has 169.589001320890476999999994 NEAR available for transfer (the total balance is 172.482461320890476999999994 NEAR, but 2.89246 NEAR is locked for storage and the transfer transaction fee is ~0.001 NEAR)
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/lKGalzAxt3zCSxOsreqdykO8X?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/lKGalzAxt3zCSxOsreqdykO8X.png" width="836"/>
</a>
</details>

#### view-ft-balance - View the balance of FT tokens

Viewing the account balance is possible at the current time (**_now_**) and at a certain moment in the past by specifying the block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

To view funds in FT tokens on the account, type the following in the terminal command line:

```txt
near tokens \
    fro_volod.testnet \
    view-ft-balance usdn.testnet \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
fro_volod.testnet account has "31942967677775774595" FT tokens (FT-contract: usdn.testnet)
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/k7Bz5r20x2Bo5RIX7Q1VnpNZC?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/k7Bz5r20x2Bo5RIX7Q1VnpNZC.png" width="836"/>
</a>
</details>

#### view-nft-assets - View the balance of NFT tokens

Viewing the account balance is possible at the current time (**_now_**) and at a certain moment in the past by specifying the block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

To view funds in NFT tokens on the account, type the following in the terminal command line:

```txt
near tokens \
    fro_volod.testnet \
    view-nft-assets paras-token-v2.testnet \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
fro_volod.testnet account has NFT tokens:
[
  {
    "approved_account_ids": {},
    "metadata": {
      "copies": 100,
      "description": null,
      "expires_at": null,
      "extra": null,
      "issued_at": "1657613801537412611",
      "media": "bafybeib65t37t2tagukok4m7f5rldfirzb5ykvdq3yqbwnbcrtllpggg6u",
      "media_hash": null,
      "reference": "bafkreidmbv4j2qylxc2mngsup7cxakw7gwyd7lu2zycznrdtqw4kc52cwu",
      "reference_hash": null,
      "starts_at": null,
      "title": "Apollo42 #01 #4",
      "updated_at": null
    },
    "owner_id": "fro_volod.testnet",
    "token_id": "1604:4"
  }
]
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/znmY5yzIlSTjOlRRRUHzeeuzJ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/znmY5yzIlSTjOlRRRUHzeeuzJ.png" width="836"/>
</a>
</details>

### contract - Manage smart-contracts: deploy code, call functions

- [call-function](#call-function---Execute-function-contract-method)
- [deploy](#deploy---Add-a-new-contract-code)
- [download-wasm](#download-wasm---Download-wasm)
- [view-storage](#view-storage---View-contract-storage-state)

#### call-function - Execute function (contract method)

- [as-read-only](#as-read-only---Calling-a-view-method)
- [as-transaction](#as-transaction---Calling-a-change-method)

##### as-read-only - Calling a view method

Viewing data is possible at the current time (**_now_**) and at a certain point in the past by specifying a block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

To run this command, type the following in the terminal command line:

```txt
near contract \
    call-function \
    as-read-only zavodil.poolv1.near get_accounts \
    json-args '{"from_index": 0, "limit": 3}' \
    network-config mainnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
[
  {
    "account_id": "zavodil.near",
    "can_withdraw": false,
    "staked_balance": "107480661091559500516766891",
    "unstaked_balance": "1307739180247557404925470405"
  },
  {
    "account_id": "gagdiez.near",
    "can_withdraw": true,
    "staked_balance": "4387193990112136827894210960",
    "unstaked_balance": "1"
  },
  {
    "account_id": "gibby49.near",
    "can_withdraw": true,
    "staked_balance": "1105950300133283278041226",
    "unstaked_balance": "1"
  }
]
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/OHhdlJEaoA4nLJSDtybgc7kCR?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/OHhdlJEaoA4nLJSDtybgc7kCR.png" width="836"/>
</a>
</details>

##### as-transaction - Calling a change method

To run this command, type the following in the terminal command line:

```txt
near contract \
    call-function \
    as-transaction turbo.volodymyr.testnet rate \
    json-args '{"other_user":"volodymyr.testnet", "vote":5}' \
    prepaid-gas '3 Tgas' \
    attached-deposit '1 NEAR' \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
The "rate" call to <turbo.volodymyr.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 7RuoSAdCctSEw63GKsfQJg1YXRzH3msUCo4oygzauPko
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/7RuoSAdCctSEw63GKsfQJg1YXRzH3msUCo4oygzauPko
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/S6LHwINBHskznxMrJPHzUmgxM?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/S6LHwINBHskznxMrJPHzUmgxM.png" width="836"/>
</a>
</details>

#### deploy - Add a new contract code

In order to add a new contract, in the terminal command line type:

```txt
near contract \
    deploy \
    262.volodymyr.testnet \
    use-file /Users/frovolod/Documents/NEAR/rust-counter/contract/target/wasm32-unknown-unknown/release/rust_counter_tutorial.wasm \
    with-init-call increment \
    json-args {} \
    prepaid-gas '1 TGas' \
    attached-deposit '0 NEAR' \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction sent ...
Successful transaction
Contract code has been successfully deployed.
The "increment" call to <262.volodymyr.testnet> on behalf of <262.volodymyr.testnet> succeeded.
Transaction ID: 4YGGhF88aevNGpF5uaXNGHfQprHRqkia7eTpyxegJVms
To see the transaction in the transaction explorer, please open this url in your browser:
https://testnet.nearblocks.io/txns/4YGGhF88aevNGpF5uaXNGHfQprHRqkia7eTpyxegJVms
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/7KD9gM9tj2AWtgGpjUmytkPg9?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/7KD9gM9tj2AWtgGpjUmytkPg9.png" width="836"/>
</a>
</details>

#### download-wasm - Download wasm

You can download the contract file for the current moment (**_now_**) and for a certain moment in the past by specifying the block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).

In order to get the contract file, type the following in the terminal command line:

```txt
near contract \
    download-wasm 262.volodymyr.testnet \
    to-folder /Users/frovolod/Downloads \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>
```txt
The file "/Users/frovolod/Downloads/contract_262_volodymyr_testnet.wasm" was downloaded successfully
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/2UbeTzLJq16qtCUR015wuRFmN?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/2UbeTzLJq16qtCUR015wuRFmN.png" width="836"/>
</a>
</details>

#### view-storage - View contract storage state

You can view the contract key values at the current moment in time (**_now_**) and at a certain point in the past by specifying a block (**_at-block-height_** or **_at-block-hash_**).  
Examples of the use of these parameters are discussed in the ([View properties for an account](#view-account-summary---view-properties-for-an-account)).  
The keys themselves can be viewed all (**_all_**) or filtered using **_keys-start-with-string_** or **_keys-start-with-bytes-as-base64_**.

To view contract keys, enter at the terminal command line:

```txt
near contract \
    view-storage turbo.volodymyr.testnet \
    all \
    as-json \
    network-config testnet \
    now
```

<details><summary><i>The result of this command will be as follows:</i></summary>
```txt
Contract state (values):
[
  {
    "key": "MjF2b2xvZHlteXIudGVzdG5ldA==",
    "value": "JwAAAAAAAAAIAAAAAAAAAA=="
  },
  {
    "key": "U1RBVEU=",
    "value": ""
  },
  {
    "key": "ZnJvX3ZvbG9kLnRlc3RuZXQ=",
    "value": "HQAAAAAAAAAGAAAAAAAAAA=="
  },
  {
    "key": "dm9sb2R5bXlyLnRlc3RuZXQ=",
    "value": "QAEAAAAAAABAAAAAAAAAAA=="
  }
]
Contract state (proof):
[]
```
</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/ylVt2VzX2GZp6nP5OccBbdKul?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ylVt2VzX2GZp6nP5OccBbdKul.png" width="836"/>
</a>
</details>

### transaction - Operate transactions

- [view-status](#view-status---View-a-transaction-status)
- [construct-transaction](#construct-transaction---Construct-a-new-transaction)
- [sign-transaction](#sign-transaction---Sign-previously-prepared-unsigned-transaction)
- [send-signed-transaction](#send-signed-transaction---Send-a-signed-transaction)
- [send-meta-transaction](#send-meta-transaction---Act-as-a-relayer-to-send-a-signed-delegate-action-meta-transaction)

#### view-status - View a transaction status

To view the status of the desired transaction, type its hash in the terminal command line:

```txt
near transaction \
    view-status GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank \
    volodymyr.testnet \
    network-config testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Transaction status: FinalExecutionOutcomeWithReceiptView {
    final_outcome: FinalExecutionOutcome {
        status: SuccessValue(``),
        transaction: SignedTransactionView {
            signer_id: AccountId(
                "volodymyr.testnet",
            ),
            public_key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS,
            nonce: 165,
            receiver_id: AccountId(
                "qweqweqwe.volodymyr.testnet",
            ),
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
                executor_id: AccountId(
                    "volodymyr.testnet",
                ),
                status: SuccessReceiptId(5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh),
                metadata: ExecutionMetadataView {
                    version: 1,
                    gas_profile: None,
                },
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
                    executor_id: AccountId(
                        "qweqweqwe.volodymyr.testnet",
                    ),
                    status: SuccessValue(``),
                    metadata: ExecutionMetadataView {
                        version: 1,
                        gas_profile: None,
                    },
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
                    executor_id: AccountId(
                        "volodymyr.testnet",
                    ),
                    status: SuccessValue(``),
                    metadata: ExecutionMetadataView {
                        version: 1,
                        gas_profile: None,
                    },
                },
            },
        ],
    },
    receipts: [
        ReceiptView {
            predecessor_id: AccountId(
                "volodymyr.testnet",
            ),
            receiver_id: AccountId(
                "qweqweqwe.volodymyr.testnet",
            ),
            receipt_id: `5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh`,
            receipt: Action {
                signer_id: AccountId(
                    "volodymyr.testnet",
                ),
                signer_public_key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS,
                gas_price: 103000000,
                output_data_receivers: [],
                input_data_ids: [],
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
            },
        },
        ReceiptView {
            predecessor_id: AccountId(
                "system",
            ),
            receiver_id: AccountId(
                "volodymyr.testnet",
            ),
            receipt_id: `851GMnZZ5FJ2aDSHM34N99yVb1ZkwY8n7F8rUcvuRpUU`,
            receipt: Action {
                signer_id: AccountId(
                    "volodymyr.testnet",
                ),
                signer_public_key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS,
                gas_price: 0,
                output_data_receivers: [],
                input_data_ids: [],
                actions: [
                    Transfer {
                        deposit: 1273665187500000000,
                    },
                ],
            },
        },
    ],
}
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/xf69gJEha7yO27E27CZszkN97?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/xf69gJEha7yO27E27CZszkN97.png" width="836"/>
</a>
</details>

#### construct-transaction - Construct a new transaction

Let's consider an example when it is necessary to perform several actions within one transaction:

1. Create an account
2. Add access keys to the created account
3. Transfer tokens to the created account

To do this, we will use the transaction constructor:

<details><summary>Demonstration of the command in interactive mode</summary>
<a href="https://asciinema.org/a/WNbxN1GB861q2sBbiKbQyVl3S?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/WNbxN1GB861q2sBbiKbQyVl3S.png" width="836"/>
</a>
</details>

#### sign-transaction - Sign previously prepared unsigned transaction

Consider an example of using the ability to create a transaction in _offline_:

1. Create a transaction.
2. When choosing how to sign a transaction, select the _sign later_ option and follow the instructions.
3. The displayed transaction in base64 format can be used here to sign it and/or send it later.

<details><summary>Demonstration of the command in interactive mode</summary>
<a href="https://asciinema.org/a/7yO1OobKvE3EWezUexPEHYYVC?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/7yO1OobKvE3EWezUexPEHYYVC.png" width="836"/>
</a>
</details>

#### send-signed-transaction - Send a signed transaction

Let's look at the previous example, using the capabilities of sending a signed transaction:

1. Create a transaction.
2. Sign the transaction with your access keys.
3. Display the transaction on the screen in base64 format.
4. Send transaction.

<details><summary>Demonstration of the command in interactive mode</summary>
<a href="https://asciinema.org/a/ignaXjJrvvDpQV4YUEK96iozX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ignaXjJrvvDpQV4YUEK96iozX.png" width="836"/>
</a>
</details>

#### send-meta-transaction - Act as a relayer to send a signed delegate action (meta-transaction)

Consider an example of using metatransaction functions:

1. Create a transaction.
2. Specify a _network_ that supports meta-transactions.
3. Sign the transaction with your access keys.
4. Display the transaction on the screen in base64 format and transfer it to the relay for sending.

Send signed delegated transaction:

<details><summary>Demonstration of the command in interactive mode</summary>
<a href="https://asciinema.org/a/79Pwj2KxIHJgxC0CFrRTgfNcs?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/79Pwj2KxIHJgxC0CFrRTgfNcs.png" width="836"/>
</a>
</details>

### config - Manage connections in a configuration file

- [show-connections](#show-connections---Show-a-list-of-network-connections)
- [add-connection](#add-connection---Add-a-network-connection)
- [delete-connection](#delete-connection---Delete-a-network-connection)

#### show-connections - Show a list of network connections

To view the data of the configuration file (_config.toml_), you can use the interactive mode or type in the terminal command line:

```txt
near config show-connections
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
credentials_home_dir = "/Users/frovolod/.near-credentials"
[network_connection.mainnet]
network_name = "mainnet"
rpc_url = "https://archival-rpc.mainnet.near.org/"
wallet_url = "https://app.mynearwallet.com/"
explorer_transaction_url = "https://nearblocks.io/txns/"
linkdrop_account_id = "near"

[network_connection.testnet]
network_name = "testnet"
rpc_url = "https://archival-rpc.testnet.near.org/"
wallet_url = "https://testnet.mynearwallet.com/"
explorer_transaction_url = "https://testnet.nearblocks.io/txns/"
linkdrop_account_id = "testnet"
faucet_url = "https://helper.nearprotocol.com/account"

[network_connection.pagoda-testnet]
network_name = "testnet"
rpc_url = "https://near-testnet.api.pagoda.co/rpc/v1/"
rpc_api_key = "c0a25b3c-39c2-4f62-a621-50e208b88e64"
wallet_url = "https://testnet.mynearwallet.com/"
explorer_transaction_url = "https://testnet.nearblocks.io/txns/"
linkdrop_account_id = "testnet"
faucet_url = "https://helper.nearprotocol.com/account"
meta_transaction_relayer_url = "https://near-testnet.api.pagoda.co/relay"
```

</details>

#### add-connection - Add a network connection

To add network details to the configuration file (_config.toml_), you can use interactive mode or type in the terminal command line:

```txt
near config \
    add-connection \
        --network-name testnet \
        --connection-name pagoda-testnet \
        --rpc-url https://near-testnet.api.pagoda.co/rpc/v1/ \
        --wallet-url https://testnet.mynearwallet.com/ \
        --explorer-transaction-url https://testnet.nearblocks.io/txns/ \
        --rpc-api-key 'c0a25b3c-39c2-4f62-a621-50e208b88e64' \
        --linkdrop-account-id testnet \
        --faucet-url https://helper.nearprotocol.com/account \
        --meta-transaction-relayer-url https://near-testnet.api.pagoda.co/relay
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Configuration data is stored in a file "/Users/frovolod/Library/Application Support/near-cli/config.toml"
Network connection "pagoda-testnet" was successfully added to config.toml
```

</details>

<details><summary><i>Demonstration of the command in interactive mode</i></summary>
<a href="https://asciinema.org/a/49s6yuDmxQyaA2XgEqlBC6cpN?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/49s6yuDmxQyaA2XgEqlBC6cpN.png" width="836"/>
</a>
</details>

#### delete-connection - Delete a network connection

To remove the network from the configuration file (_config.toml_), you can use interactive mode or type in the terminal command line:

```txt
near config delete-connection pagoda-testnet
```

<details><summary><i>The result of this command will be as follows:</i></summary>

```txt
Configuration data is stored in a file "/Users/frovolod/Library/Application Support/near-cli/config.toml"
Network connection "pagoda-testnet" was successfully removed from config.toml
```

</details>
