## Инструкция

Это руководство предназначено для того, чтобы дать подробное описание утилиты _near CLI_ и
обзор её возможностей. Предполагается, что утилита _near CLI_
[установлена](README.ru.md#installation)
и пользователи знакомы с использованием инструментов командной строки. Также предполагается Unix-подобная система, хотя большинство команд, вероятно, легко переводимы в любую среду оболочки командной строки.

Спомощью _near CLI_ можно создать, подписать и отправить транзакцию в режиме _online_, который включен по умолчанию.
В режиме _offline_ можно создать и подписать транзакцию. Транзакция, кодированная в base64 может быть [подписана](#sign-transaction---sign-previously-prepared-unsigned-transaction) или [отправлена](#send-signed-transaction---send-a-signed-transaction) позже (даже с другого компьютера). Для входа в режим _offline_ необходимо в команде установить флаг ```--offline```:
```txt
near --offline tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-later
```

_near CLI_ с помощью флага `--quiet` подавляет вывод лишней информации:
```txt
near --quiet tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

_near CLI_ — отличный инструмент для глубокого понимания NEAR. Например, если вы хотите просмотреть более подробную информацию о выполняемых вызовах RPC и их параметрах, просто запустите CLI с флагом `--teach-me`:
```txt
near --teach-me tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Unsigned transaction:
 |    signer_id:    fro_volod.testnet
 |    receiver_id:  volodymyr.testnet
 |    actions:
 |       -- transfer deposit:    0.1 NEAR
 INFO Signing the transaction with a key saved in the secure keychain ...:Getting a list of: fro_volod.testnet access keys ...
 INFO I am making HTTP call to NEAR JSON RPC to get a list of keys for `fro_volod.testnet` account, learn more https://docs.near.org/api/rpc/access-keys#view-access-key-list
 INFO HTTP POST https://archival-rpc.testnet.near.org/
 INFO JSON Body:
 |    {
 |      "id": "RSDcGn4WP",
 |      "jsonrpc": "2.0",
 |      "method": "query",
 |      "params": {
 |        "account_id": "fro_volod.testnet",
 |        "finality": "final",
 |        "request_type": "view_access_key_list"
 |      }
 |    }
 INFO JSON RPC Response:
 |    {
 |      "block_hash": "DaoWCSVSMVS6d5rLsYBgVKwSKb8XxZWN2KpEg2dQEbEY",
 |      "block_height": 169978024,
 |      "keys": [
 |        {
 |          "access_key": {
 |            "nonce": 116133598000035,
 |            "permission": "FullAccess"
 |          },
 |          "public_key": "ed25519:1TprKa4burMqDMjDHyBSUaFQQczF7NamhxTx2yEXe9P"
 |        },
 |        {
 |          "access_key": {
 |            "nonce": 94982716000000,
 |            "permission": {
 |              "FunctionCall": {
 |                "allowance": "250000000000000000000000",
 |                "method_names": [],
 |                "receiver_id": "mintspace2.testnet"
 |              }
 |            }
 |          },
 |          "public_key": "ed25519:7YCfA1KrToJtAYGTBgAMe4LWfQEi4iwLGcH2q5SvGKzD"
 |        },
 |        {
 |          "access_key": {
 |            "nonce": 147781057000109,
 |            "permission": "FullAccess"
 |          },
 |          "public_key": "ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx"
 |        },
 |        {
 |          "access_key": {
 |            "nonce": 101493245000000,
 |            "permission": {
 |              "FunctionCall": {
 |                "allowance": "10000000000000000000000000",
 |                "method_names": [
 |                  "set_a",
 |                  "set_b"
 |                ],
 |                "receiver_id": "meta.pool.testnet"
 |              }
 |            }
 |          },
 |          "public_key": "ed25519:8KHRkmpWbAp6wHZ5imAGFZzRAHzha2cZoz7cc3J42Bz8"
 |        },
 |        {
 |          "access_key": {
 |            "nonce": 98944792000000,
 |            "permission": "FullAccess"
 |          },
 |          "public_key": "ed25519:8dGkLiLD285Pzgp6v4mhaUbJyFvwEMvzjss1u9xZokTz"
 |        },
 |        {
 |          "access_key": {
 |            "nonce": 105032344000005,
 |            "permission": "FullAccess"
 |          },
 |          "public_key": "ed25519:J5uajy3m24sEQdw1uWA5kD2i3PDcxRxcFYotVZqRyrm6"
 |        }
 |      ]
 |    }
 INFO Signing the transaction with a key saved in the secure keychain ...:Trying to sign with the legacy keychain ...:Signing the transaction with a key saved in legacy keychain ...:Getting access key information:: ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx on fro_volod.testnet account ...
 INFO I am making HTTP call to NEAR JSON RPC to get an access key ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx details on `fro_volod.testnet` account, learn more https://docs.near.org/api/rpc/access-keys#view-access-key
 INFO HTTP POST https://archival-rpc.testnet.near.org/
 INFO JSON Body:
 |    {
 |      "id": "3DCGHrjRK",
 |      "jsonrpc": "2.0",
 |      "method": "query",
 |      "params": {
 |        "account_id": "fro_volod.testnet",
 |        "finality": "optimistic",
 |        "public_key": "ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx",
 |        "request_type": "view_access_key"
 |      }
 |    }
 INFO JSON RPC Response:
 |    {
 |      "block_hash": "BDT76QHmdMC5yKHBuJBi3vgAC1j4hPSHoX5oVFpx1SG2",
 |      "block_height": 169978028,
 |      "nonce": 147781057000109,
 |      "permission": "FullAccess"
 |    }
 INFO Your transaction was signed successfully.
 |    Public key: ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx
 |    Signature:  ed25519:4r8YNLMkqhxSTFLejMf8JvZw6q8ue9BuQHf7JEycamAWCqLckfE5zNG7ceWoUfagQaJLTunD59ig4LuecYyVk8Qe

 INFO Sending transaction ...:Broadcasting transaction via RPC: https://archival-rpc.testnet.near.org/
 INFO I am making HTTP call to NEAR JSON RPC to broadcast a transaction, learn more https://docs.near.org/api/rpc/transactions#send-tx
 INFO HTTP POST https://archival-rpc.testnet.near.org/
 INFO JSON Body:
 |    {
 |      "id": "1ARLaDA3J",
 |      "jsonrpc": "2.0",
 |      "method": "broadcast_tx_commit",
 |      "params": [
 |        "EQAAAGZyb192b2xvZC50ZXN0bmV0AGYjuraPK0UBuPn9vFOErFp7IcGKqhQ5AqH8v2LHNdzhrjJo9WeGAAARAAAAdm9sb2R5bXlyLnRlc3RuZXSXxVsmKUVo0lmnzQ013O+bqjznbnB5g/3biI+j62VuOwEAAAADAACA9krhxwItFQAAAAAAAADAazoMP0kIphzt/zQ7Z97rr64FLGGsXMJDS8sXpuX8WwQdEgF6GZX+fz8hvKx5GqB4nrxnwrxbZScTQcs9mcEP"
 |      ]
 |    }
 INFO JSON RPC Response:
 |    {
 |      "receipts_outcome": [
 |        {
 |          "block_hash": "7rKeJTzfty8YKWmxjCSAi3YB3AYkTs6b6BbrN3TfbzSu",
 |          "id": "HgDAs4D8SCe7RxgH9Knomg99N9LF92b7nUCC6FRbC7Eo",
 |          "outcome": {
 |            "executor_id": "volodymyr.testnet",
 |            "gas_burnt": 223182562500,
 |            "logs": [],
 |            "metadata": {
 |              "gas_profile": [],
 |              "version": 3
 |            },
 |            "receipt_ids": [
 |              "5LTPntJ4CDmnHCjb4URXqmnCxudBhmPHDv7kpuEWY8U4"
 |            ],
 |            "status": {
 |              "SuccessValue": ""
 |            },
 |            "tokens_burnt": "22318256250000000000"
 |          },
 |          "proof": [
 |            {
 |              "direction": "Right",
 |              "hash": "3sWdNsYk1wmbuQBJtWFuxVTViYjhqHrd7oahLAtGK6xC"
 |            }
 |          ]
 |        }
 |      ],
 |      "status": {
 |        "SuccessValue": ""
 |      },
 |      "transaction": {
 |        "actions": [
 |          {
 |            "Transfer": {
 |              "deposit": "100000000000000000000000"
 |            }
 |          }
 |        ],
 |        "hash": "F3eZmhtFekCrzKMbc3uk5UbKkMsuuecj6WbK9spcz8bW",
 |        "nonce": 147781057000110,
 |        "public_key": "ed25519:7siBhHN2eYNCubz5jAJhMdo34x33QJt5ZgUJBTNifZAx",
 |        "receiver_id": "volodymyr.testnet",
 |        "signature": "ed25519:4r8YNLMkqhxSTFLejMf8JvZw6q8ue9BuQHf7JEycamAWCqLckfE5zNG7ceWoUfagQaJLTunD59ig4LuecYyVk8Qe",
 |        "signer_id": "fro_volod.testnet"
 |      },
 |      "transaction_outcome": {
 |        "block_hash": "4Ctk97bpxgY3npmU41n5t7ZviKcVDD2sK6N9E1RvanER",
 |        "id": "F3eZmhtFekCrzKMbc3uk5UbKkMsuuecj6WbK9spcz8bW",
 |        "outcome": {
 |          "executor_id": "fro_volod.testnet",
 |          "gas_burnt": 223182562500,
 |          "logs": [],
 |          "metadata": {
 |            "gas_profile": null,
 |            "version": 1
 |          },
 |          "receipt_ids": [
 |            "HgDAs4D8SCe7RxgH9Knomg99N9LF92b7nUCC6FRbC7Eo"
 |          ],
 |          "status": {
 |            "SuccessReceiptId": "HgDAs4D8SCe7RxgH9Knomg99N9LF92b7nUCC6FRbC7Eo"
 |          },
 |          "tokens_burnt": "22318256250000000000"
 |        },
 |        "proof": [
 |          {
 |            "direction": "Right",
 |            "hash": "2ktmkisPC2M6uXFKc6XuAWGA1WbtewS2L6ugkLv92K6T"
 |          },
 |          {
 |            "direction": "Right",
 |            "hash": "HLHeyozXBSqN7Tz1JV3bxQ8J9z9dhAUSbKc5tXHDzHh2"
 |          }
 |        ]
 |      }
 |    }
 INFO 
 |    --- Logs ---------------------------
 |    Logs [volodymyr.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    <fro_volod.testnet> has transferred 0.1 NEAR to <volodymyr.testnet> successfully.

 |    Gas burned: 0.447 Tgas
 |    Transaction fee: 0.0000446365125 NEAR (approximately $0.00015176 USD, using $3.40 USD/NEAR exchange rate)
 |    Transaction ID: 8WEG4LgrpEbyhbhHqUJcJ9QT3rqccYHfijmUVL7uPj6a
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions8WEG4LgrpEbyhbhHqUJcJ9QT3rqccYHfijmUVL7uPj6a


Here is your console command if you need to script it or re-run:
    ./target/debug/near --teach-me tokens fro_volod.testnet send-near volodymyr.testnet '0.1 NEAR' network-config testnet sign-with-keychain send
```
</details>

Прежде, чем перейти к описанию конкретных команд, необходимо рассмотреть два общих для этих команд пункта:

1. Подпись транзакции

   _near CLI_ предполагает несколько способов подписи созданной транзакции. Рассмотрим подробнее каждый.

    - _sign-with-keychain - Sign the transaction with a key saved in the secure keychain_

        _near CLI_ позволяет хранить и извлекать пароли базовом безопасном хранилище, специфичном для конкретной ОС. В этом хранилище _near CLI_ самостоятельно найдет ключи доступа и подпишет созданную транзакцию.

    - _sign-with-legacy-keychain - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)_

        _near CLI_ самостоятельно найдет ключи доступа и подпишет созданную транзакцию.
        Каталог с ключами доступа определен в [конфигурационном файле](#config---manage-connections-in-a-configuration-file).
        Ключи доступа должны находиться в файле _публичный-ключ.json_, расположенном в _/Users/user/.near-credentials/имя-сети/имя-пользователя/_.
        Например, _/Users/frovolod/.near-credentials/testnet/volodymyr.testnet/ed25519_8h7kFK4quSUJRkUwo3LLiK83sraEm2jnQTECuZhWu8HC.json_

        <details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
        <a href="https://asciinema.org/a/SAlkUVFzRth0ifbx3wJt9aZ0C?autoplay=1&t=1&speed=2">
            <img src="https://asciinema.org/a/SAlkUVFzRth0ifbx3wJt9aZ0C.png" width="836"/>
        </a>
        </details>

    - _sign-with-ledger - Sign the transaction with Ledger Nano device_

        Этот вариант предполагает подписание созданной транзакции при помощи леджера.

    - _sign-with-plaintext-private-key - Sign the transaction with a plaintext private key_

        При выборе этого варианта подписи _near CLI_ попросит пользователя ввести ключи доступа:
        - "public_key":"ed25519:Ebx7...",
        - "private_key":"ed25519:2qM8..."

    - _sign-with-access-key-file - Sign the transaction using the account access key file (access-key-file.json)_

        При выборе этого варианта подписи _near CLI_ попросит пользователя ввести путь к файлу, в котором находится информация о ключах доступа к аккаунту.

    - _sign-with-seed-phrase - Sign the transaction using the seed phrase_

        При выборе этого варианта подписи _near CLI_ попросит пользователя ввести мнемоническую фразу, связанную с аккаунтом.

    - _sign-later - Prepare unsigned transaction (we'll use base64 encoding to simplify copy-pasting)_

        Этот вариант предполагает подписание созданной транзакции [позже](#sign-transaction---sign-previously-prepared-unsigned-transaction).


2. Действия с подписанной транзакцией

    _near CLI_ поддерживает мета-транзакции, описанные в спецификации [NEP-366](https://near.github.io/nearcore/architecture/how/meta-tx.html#meta-transactions). Для её создания достаточно указать _network_, поддерживающую мета-транзакции. Узнать о такой поддержке можно в [конфигурационном файле](#show-connections---Show-a-list-of-network-connections). За возможность поддержки мета-транзакции отвечает поле *meta_transaction_relayer_url*. Например:
    ```txt
    meta_transaction_relayer_url = "https://near-testnet.api.pagoda.co/relay"
    ```

    Подписанную транзакцию / мета-транзакцию можно либо немедленно отправить на выполнение:

   - _send - Send the transaction to the network_

   либо вывести на экран в формате base64 для последующей отправки:

   - _display - Print only the signed transaction in base64 encoding. We will use it to send it later. ([Example](#send-signed-transaction---send-a-signed-transaction): near transaction send-signed-transaction 'EQAAAHZvb...' ...)_

### Группы команд

- [account     - Manage accounts](#account---Manage-accounts)
- [tokens      - Manage token assets such as NEAR, FT, NFT](#tokens---Manage-token-assets-such-as-NEAR-FT-NFT)
- [staking     - Manage staking: view, add and withdraw stake](#staking---Manage-staking-view-add-and-withdraw-stake)
- [contract    - Manage smart-contracts: deploy code, call functions](#contract---Manage-smart-contracts-deploy-code-call-functions)
- [transaction - Operate transactions](#transaction---Operate-transactions)
- [config      - Manage connections in a configuration file](#config---Manage-connections-in-a-configuration-file)

### account - Manage accounts

Просмотреть сведения об аккаунте ([View properties for an account](#view-account-summary---view-properties-for-an-account)) и просмотреть ключи доступа к аккаунту ([View a list of access keys of an account](#list-keys---View-a-list-of-access-keys-of-an-account)) возможно на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***). На примерах ниже показаны варианты применения этих режимов.

- [view-account-summary](#view-account-summary---View-properties-for-an-account)
- [import-account](#import-account---Import-existing-account-aka-sign-in)
- [export-account](#export-account---Export-existing-account)
- [create-account](#create-account---Create-a-new-account)
- [update-social-profile](#update-social-profile---Update-NEAR-Social-profile)
- [delete-account](#delete-account---Delete-an-account)
- [list-keys](#list-keys---View-a-list-of-access-keys-of-an-account)
- [get-public-key](#get-public-key---Get-the-public-key-to-your-account)
- [add-key](#add-key---Add-an-access-key-to-an-account)
- [delete-keys](#delete-keys---Delete-access-keys-from-an-account)
- [manage-storage-deposit](#manage-storage-deposit---Storage-management-deposit-withdrawal-balance-review)

#### view-account-summary - View properties for an account

- [now](#now---View-properties-in-the-final-block)
- [at-block-height](#at-block-height---View-properties-in-a-height-selected-block)
- [at-block-hash](#at-block-hash---View-properties-in-a-hash-selected-block)

##### now - View properties in the final block

Для просмотра сведений об аккаунте на последнем блоке необходимо ввести в командной строке терминала:

```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
---------------------------------------------------------------------------------------------------------------
 fro_volod.testnet                At block #188408667 
                                  (4wDcAgoktL85KKsMKP9oBJCUvLtsU9prNXorDJtzup27) 
---------------------------------------------------------------------------------------------------------------
 NEAR Social profile unavailable  The profile can be edited at https://near.social 
                                  or using the cli command: bos social-db manage-profile 
                                  (https://github.com/bos-cli-rs/bos-cli-rs) 
---------------------------------------------------------------------------------------------------------------
 Native account balance           3076.37 NEAR 
---------------------------------------------------------------------------------------------------------------
 Validator stake                  0 NEAR 
---------------------------------------------------------------------------------------------------------------
 Delegated stake                  handler error: [State of contract pool.f863973.m0 is too large to be viewed] 
---------------------------------------------------------------------------------------------------------------
 Storage used by the account      295.1 KB 
---------------------------------------------------------------------------------------------------------------
 Contract (SHA-256 checksum hex)  fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec 
---------------------------------------------------------------------------------------------------------------
 Access keys                      50 full access keys and 25 function-call-only access keys 
---------------------------------------------------------------------------------------------------------------
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/yx1X3lSBI2LDH74MVau8O9AqX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/yx1X3lSBI2LDH74MVau8O9AqX.png" width="836"/>
</a>
</details>

##### at-block-height - View properties in a height-selected block

Для просмотра сведений об аккаунте на конктретном блоке можно указать высоту данного блока. Для этого нужно ввести в командной строке терминала:
```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    at-block-height 73069245
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
------------------------------------------------------------------------------------------------------------------
 fro_volod.testnet                At block #73069245 
                                  (HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY) 
------------------------------------------------------------------------------------------------------------------
 NEAR Social profile unavailable  The profile can be edited at https://near.social 
                                  or using the cli command: bos social-db manage-profile 
                                  (https://github.com/bos-cli-rs/bos-cli-rs) 
------------------------------------------------------------------------------------------------------------------
 Native account balance           199.00 NEAR 
------------------------------------------------------------------------------------------------------------------
 Validator stake                  0 NEAR 
------------------------------------------------------------------------------------------------------------------
 Delegated stake                  handler error: [account 4ire-pool.pool.f863973.m0 does not exist while viewing] 
------------------------------------------------------------------------------------------------------------------
 Storage used by the account      288.7 KB 
------------------------------------------------------------------------------------------------------------------
 Contract (SHA-256 checksum hex)  fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec 
------------------------------------------------------------------------------------------------------------------
 Access keys                      12 full access keys and 0 function-call-only access keys 
------------------------------------------------------------------------------------------------------------------
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/DR8EApNOLXWEYox2v4P3JnQbL?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/DR8EApNOLXWEYox2v4P3JnQbL.png" width="836"/>
</a>
</details>

##### at-block-hash - View properties in a hash-selected block

Для просмотра сведений об аккаунте необходимо ввести в командной строке терминала:
```txt
near account \
    view-account-summary fro_volod.testnet \
    network-config testnet \
    at-block-hash HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY
````

<details><summary><i>Результат выполнения команды</i></summary>

```txt
---------------------------------------------------------------------------------------------------------------
 fro_volod.testnet                At block #73069245 
                                  (HCUJq3vQ3ztyCZAhmRmHR3cwSDcoE4zEbaWkhAjFuxUY) 
---------------------------------------------------------------------------------------------------------------
 NEAR Social profile unavailable  The profile can be edited at https://near.social 
                                  or using the cli command: bos social-db manage-profile 
                                  (https://github.com/bos-cli-rs/bos-cli-rs) 
---------------------------------------------------------------------------------------------------------------
 Native account balance           199.00 NEAR 
---------------------------------------------------------------------------------------------------------------
 Validator stake                  0 NEAR 
---------------------------------------------------------------------------------------------------------------
 Delegated stake                  handler error: [State of contract pool.f863973.m0 is too large to be viewed] 
---------------------------------------------------------------------------------------------------------------
 Storage used by the account      288.7 KB 
---------------------------------------------------------------------------------------------------------------
 Contract (SHA-256 checksum hex)  fd999145baf49ece7d09fca7d030d384c4ea8ed4df651c6e87a015c4dfa6c0ec 
---------------------------------------------------------------------------------------------------------------
 Access keys                      12 full access keys and 0 function-call-only access keys 
---------------------------------------------------------------------------------------------------------------
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/m8N04Nw1ZTjKSjWxDuSWQRRQF?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/m8N04Nw1ZTjKSjWxDuSWQRRQF.png" width="836"/>
</a>
</details>

#### import-account - Import existing account (a.k.a. "sign in")

- [using-web-wallet](#using-web-wallet---Import-existing-account-using-NEAR-Wallet-aka-sign-in)
- [using-seed-phrase](#using-seed-phrase---Import-existing-account-using-a-seed-phrase)
- [using-private-key](#using-private-key---Import-existing-account-using-a-private-key)


#### using-web-wallet - Import existing account using NEAR Wallet (a.k.a. "sign in")

Для авторизации пользователя необходимо ввести в командной строке терминала:
```txt
near account \
    import-account \
    using-web-wallet \
    network-config testnet
```

Вы будете перенаправлены браузер для авторизации.
По умолчанию - это https://app.mynearwallet.com/ (для testnet - https://testnet.mynearwallet.com/). Но вы можете изменить адрес для авторизации с помощью флага `--wallet-url`:
```txt
near account \
    import-account \
    using-web-wallet \
    network-config testnet\
    --wallet-url 'https://wallet.testnet.near.org/'
```

После успешной авторизации в _[NEAR Wallet](https://wallet.near.org/)_ необходимо вернуться в терминал и ввести имя пользователя.
<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    The data for the access key is saved in the keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/7NfUlDCVzSOyRMyK4WznHl9OR?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/7NfUlDCVzSOyRMyK4WznHl9OR.png" width="836"/>
</a>
</details>

#### using-seed-phrase - Import existing account using a seed phrase

Для авторизации пользователя необходимо ввести в командной строке терминала:
```txt
near account \
    import-account \
    using-seed-phrase 'trigger arrow grunt vendor crane safe reflect please sponsor verify club shiver' \
        --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    The data for the access key is saved in the keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/A6Nl0T1RzCWxiKssA35EFXzoJ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/A6Nl0T1RzCWxiKssA35EFXzoJ.png" width="836"/>
</a>
</details>

#### using-private-key - Import existing account using a private key

Для авторизации пользователя необходимо ввести в командной строке терминала:
```txt
near account \
    import-account \
    using-private-key ed25519:3AoMxLat91aAdkh4vyq7MgbKepYhSiC5WzknLFbiXUKfsoCXXeuN9W6R4EpFd3TLvBms7gbafupvtvQJmBt7W24f \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    The file: /Users/frovolod/.near-credentials/testnet/volodymyr.testnet/ed25519_3fm1ctizEANiJG2CgJXx41e18BjtNTAnB4hfYSMjd4Fh.json already exists! Therefore it was not overwritten.
 |    The file: /Users/frovolod/.near-credentials/testnet/volodymyr.testnet.json already exists! Therefore it was not overwritten.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/HOAvsRMGyf2ZCm88i1rc9xh5Q?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/HOAvsRMGyf2ZCm88i1rc9xh5Q.png" width="836"/>
</a>
</details>

#### export-account - Export existing account

- [using-web-wallet](#using-web-wallet---Export-existing-account-using-NEAR-Wallet)
- [using-seed-phrase](#using-seed-phrase---Export-existing-account-using-a-seed-phrase)
- [using-private-key](#using-private-key---Export-existing-account-using-a-private-key)


#### using-web-wallet - Export existing account using NEAR Wallet

Для экспорта существующего аккаунта необходимо ввести в командной строке терминала:
```txt
near account \
    export-account volodymyr.testnet \
    using-web-wallet \
    network-config testnet
```

Вы будете перенаправлены браузер.
По умолчанию - это https://app.mynearwallet.com/ (для testnet - https://testnet.mynearwallet.com/). Но вы можете изменить адрес для авторизации с помощью флага `--wallet-url`:
```txt
near account \
    export-account volodymyr.testnet \
    using-web-wallet \
    network-config testnet\
    --wallet-url 'https://wallet.testnet.near.org/'
```
<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/t0D7wymkkQmI4RWjjlRDIW9ri?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/t0D7wymkkQmI4RWjjlRDIW9ri.png" width="836"/>
</a>
</details>

#### using-seed-phrase - Export existing account using a seed phrase

Для экспорта существующего аккаунта необходимо ввести в командной строке терминала:
```txt
near account \
    export-account volodymyr.testnet \
    using-seed-phrase \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Here is the secret recovery seed phrase for account <volodymyr.testnet>: "feature army carpet ..." (HD Path: m/44'/397'/0').
```
</details>

#### using-private-key - Export existing account using a private key

Для экспорта существующего аккаунта необходимо ввести в командной строке терминала:
```txt
near account \
    export-account volodymyr.testnet \
    using-private-key \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Here is the private key for account <volodymyr.testnet>: ed25519:4TKr1c7p...y7p8BvGdB
```
</details>

#### create-account - Create a new account

- sponsor-by-linkdrop (Находится в разработке)
- [sponsor-by-faucet-service](#sponsor-by-faucet-service---I-would-like-the-faucet-service-sponsor-to-cover-the-cost-of-creating-an-account-testnet-only-for-now)
- [fund-myself](#fund-myself---I-would-like-fund-myself-to-cover-the-cost-of-creating-an-account)
- [fund-later](#fund-later---Create-an-implicit-account)

#### sponsor-by-faucet-service - I would like the faucet service sponsor to cover the cost of creating an account (testnet only for now)

С помощью этой команды можно создать аккаунт при помощи вспомогательного сервиса, который может спонсировать создание учетной записи (пока только testnet).
При добавлении собственной сети в конфигураторе [add-connection](#add-connection---Add-a-network-connection) можете указать свой сервис в поле *faucet_url*.
Ключи доступа к создаваемому аккаунту можно добавить несколькими способами:
- [autogenerate-new-keypair](#autogenerate-new-keypair---Automatically-generate-a-key-pair)
- [use-manually-provided-seed-prase](#use-manually-provided-seed-prase---Use-the-provided-seed-phrase-manually)
- [use-manually-provided-public-key](#use-manually-provided-public-key---Use-the-provided-public-key-manually)
- [use-ledger](#use-ledger---Use-a-ledger)

##### autogenerate-new-keypair - Automatically generate a key pair

Для создания аккаунта необходимо ввести в командной строке терминала:
```txt
near account \
    create-account sponsor-by-faucet-service test_fro.testnet \
    autogenerate-new-keypair \
    save-to-keychain \
    network-config testnet \
    create
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    New account <test_fro.testnet> created successfully.
 |    The data for the access key is saved in the keychain

 INFO 
 |    Transaction ID: DsA3CKDg1LhNg3mJufDLqAcbqrVJdqBhmisBfGmevB9M
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsDsA3CKDg1LhNg3mJufDLqAcbqrVJdqBhmisBfGmevB9M
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/R4904WX4yzroxMvQyx2RKjxAe?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/R4904WX4yzroxMvQyx2RKjxAe.png" width="836"/>
</a>
</details>

##### use-manually-provided-seed-prase - Use the provided seed phrase manually

Данная команда добавляет аккаунту заранее известную мнемоническую фразу.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account sponsor-by-faucet-service test_fro1.testnet \
    use-manually-provided-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
    network-config testnet \
    create
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    New account <test_fro1.testnet> created successfully.

 INFO 
 |    Transaction ID: DLUXKWd2bBxWYfWxXoVPu75UtBXEw9ivUdFb88MNtFyd
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsDLUXKWd2bBxWYfWxXoVPu75UtBXEw9ivUdFb88MNtFyd
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/mpQfajE66XGuoSYniCGXo2auX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/mpQfajE66XGuoSYniCGXo2auX.png" width="836"/>
</a>
</details>

##### use-manually-provided-public-key - Use the provided public key manually

Данная команда добавляет аккаунту заранее известный публичный ключ доступа.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account sponsor-by-faucet-service test_fro2.testnet \
    use-manually-provided-public-key ed25519:HVPgAsZkZ7cwLZDqK313XJsDyqAvgBxrATcD7VacA8KE \
    network-config testnet \
    create
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    New account <test_fro2.testnet> created successfully.

 INFO 
 |    Transaction ID: zTjfXq8743AF8LWjzqxGtierA5oAF39fA8eKoyEHxnc
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionszTjfXq8743AF8LWjzqxGtierA5oAF39fA8eKoyEHxnc
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/zYuBxa8EOJQ80AGTbdt3n8Wgi?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/zYuBxa8EOJQ80AGTbdt3n8Wgi.png" width="836"/>
</a>
</details>

##### use-ledger - Use a ledger

Данная команда с помощью леджера добавляет ключи доступа аккаунту.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account sponsor-by-faucet-service test_fro3.testnet \
    use-ledger \
    network-config testnet \
    create
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    New account <test_fro3.testnet> created successfully.

 INFO 
 |    Transaction ID: 6cLee6K73jV9itZrtHv55AJUyT4egu289digLjqyrdB8
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions6cLee6K73jV9itZrtHv55AJUyT4egu289digLjqyrdB8
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/b0v4IhRZRxoJ91bVcPoCfe2yl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/b0v4IhRZRxoJ91bVcPoCfe2yl.png" width="836"/>
</a>
</details>

#### fund-myself - I would like fund myself to cover the cost of creating an account

С помощью этой команды можно создать как суб-аккаунт, так и аккаунт с коротким именем, например, alice.near или alice.testnet (в сети testnet).
Ключи доступа к создаваемому аккаунту можно добавить несколькими способами:
- [autogenerate-new-keypair](#autogenerate-new-keypair---Automatically-generate-a-key-pair-fund-myself)
- [use-manually-provided-seed-prase](#use-manually-provided-seed-prase---Use-the-provided-seed-phrase-manually-fund-myself)
- [use-manually-provided-public-key](#use-manually-provided-public-key---Use-the-provided-public-key-manually-fund-myself)
- [use-ledger](#use-ledger---Use-a-ledger-fund-myself)

##### autogenerate-new-keypair - Automatically generate a key pair (fund-myself)

Для создания суб-аккаунта необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [new.fro_volod.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    New account <new.fro_volod.testnet> has been successfully created.
 |    <fro_volod.testnet> has transferred 1 NEAR to <new.fro_volod.testnet> successfully.
 |    Added access key = ed25519:9E6cc5kQUCFWnE3WLVsCcQEupXdsGT825kVEenWRjSBa to new.fro_volod.testnet.

 |    Gas burned: 8.4 Tgas
 |    Transaction fee: 0.0008349895375 NEAR (approximately $0.00276381 USD, using $3.31 USD/NEAR exchange rate)
 |    Transaction ID: CSxoCxwU5D7UQGgqEe3xUcdQCWj76PZUbga6HHXUkJiw
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsCSxoCxwU5D7UQGgqEe3xUcdQCWj76PZUbga6HHXUkJiw

 INFO 
 |    The data for the access key is saved in the keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/KnP0AE3YaZqlawk8PGCEjZLUI?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KnP0AE3YaZqlawk8PGCEjZLUI.png" width="836"/>
</a>
</details>

Для создания аккаунта с коротким именем необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [testnet]:   No logs
 |    Logs [new7.testnet]:   No logs
 |    Logs [fro_volod.testnet]:   No logs
 |    Logs [testnet]:   No logs
 |    Logs [fro_volod.testnet]:   No logs
 |    --- Result -------------------------
 |    true
 |    ------------------------------------

 |    The "create_account" call to <testnet> on behalf of <fro_volod.testnet> succeeded.

 |    Gas burned: 12.8 Tgas
 |    Transaction fee: 0.0012273219166046 NEAR (approximately $0.00353468 USD, using $2.88 USD/NEAR exchange rate)
 |    Transaction ID: EhT2qMgQ2jusMgfMzBJEiKvPxtfLchGFYFGjApuBnpvE
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsEhT2qMgQ2jusMgfMzBJEiKvPxtfLchGFYFGjApuBnpvE

 INFO 
 |    The data for the access key is saved in the keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/japVKNYt3uxjpvrijc2TkYPyi?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/japVKNYt3uxjpvrijc2TkYPyi.png" width="836"/>
</a>
</details>

##### use-manually-provided-seed-prase - Use the provided seed phrase manually (fund-myself)

Данная команда добавляет аккаунту заранее известную мнемоническую фразу.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account fund-myself seed.volodymyr.testnet '0.1 NEAR' \
    use-manually-provided-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
New account <seed.volodymyr.testnet> created successfully.
Transaction ID: 31iA2SsxtrRzb3fD5KtsFTZni8yUi2iZboNQih9bZuDt
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/31iA2SsxtrRzb3fD5KtsFTZni8yUi2iZboNQih9bZuDt
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/OV2uJcTxoUS4xsjw2qSHMSBjk?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/OV2uJcTxoUS4xsjw2qSHMSBjk.png" width="836"/>
</a>
</details>

##### use-manually-provided-public-key - Use the provided public key manually (fund-myself)

Данная команда добавляет аккаунту заранее известный публичный ключ доступа.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account fund-myself pk.volodymyr.testnet '0.1 NEAR' \
    use-manually-provided-public-key ed25519:HVPgAsZkZ7cwLZDqK313XJsDyqAvgBxrATcD7VacA8KE \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [pk.volodymyr.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    New account <pk.volodymyr.testnet> has been successfully created.
 |    <volodymyr.testnet> has transferred 0.1 NEAR to <pk.volodymyr.testnet> successfully.
 |    Added access key = ed25519:HVPgAsZkZ7cwLZDqK313XJsDyqAvgBxrATcD7VacA8KE to pk.volodymyr.testnet.

 |    Gas burned: 8.4 Tgas
 |    Transaction fee: 0.0008349895375 NEAR (approximately $0.00240476 USD, using $2.88 USD/NEAR exchange rate)
 |    Transaction ID: CMjUG79xuGVY4LuEKV1ZH1mwhEsqNVM3PxHu5FMTvAVh
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsCMjUG79xuGVY4LuEKV1ZH1mwhEsqNVM3PxHu5FMTvAVh
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/Q1o78gXKPMlysjd54z13ILq29?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/Q1o78gXKPMlysjd54z13ILq29.png" width="836"/>
</a>
</details>

##### use-ledger - Use a ledger (fund-myself)

Данная команда с помощью леджера добавляет ключи доступа аккаунту.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account fund-myself ledger1.volodymyr.testnet '0.1 NEAR' \
    use-ledger \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [ledger1.volodymyr.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    New account <ledger1.volodymyr.testnet> has been successfully created.
 |    <volodymyr.testnet> has transferred 0.1 NEAR to <ledger1.volodymyr.testnet> successfully.
 |    Added access key = ed25519:FsRjjvkQZbwcBooXyuz4WMxXtxEKLJVJ6nc3CnaurdRr to ledger1.volodymyr.testnet.

 |    Gas burned: 8.4 Tgas
 |    Transaction fee: 0.0008349895375 NEAR (approximately $0.00238807 USD, using $2.86 USD/NEAR exchange rate)
 |    Transaction ID: E8V5rKKZXBhJc11zyXjs3HnrtbL8SWduogAi2NHUQtvy
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsE8V5rKKZXBhJc11zyXjs3HnrtbL8SWduogAi2NHUQtvy
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/90UD5uLHp2A4cWAF4yg3nFycX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/90UD5uLHp2A4cWAF4yg3nFycX.png" width="836"/>
</a>
</details>

#### fund-later - Create an implicit-account

- [use-auto-generation](#use-auto-generation---Use-auto-generation-to-create-an-implicit-account)
- [use-ledger](#use-ledger---Use-ledger-to-create-an-implicit-account)
- [use-seed-phrase](#use-seed-phrase---Use-seed-phrase-to-create-an-implicit-account)

##### use-auto-generation - Use auto-generation to create an implicit account

Данная команда автоматически генерирует ключи доступа и сохраняет их в файле с именем _implicit-account-id_.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account \
    fund-later \
    use-auto-generation \
    save-to-folder /Users/frovolod/.near-credentials/implicit
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO The file "/Users/frovolod/.near-credentials/implicit/58dc6259c521584ae83a790e6a540671330b0942d30e1aa96716b50d0df90427.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/jxAWqa9i8flsU82lLbjeXWxYJ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/jxAWqa9i8flsU82lLbjeXWxYJ.png" width="836"/>
</a>
</details>

##### use-ledger - Use ledger to create an implicit account

Данная команда с помощью леджера генерирует ключи доступа и сохраняет их в файле с именем _implicit-account-id_.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account \
    fund-later \
    use-ledger \
    save-to-folder /Users/frovolod/.near-credentials/implicit/ledger
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO The file "/Users/frovolod/.near-credentials/implicit/dceea0a5598a57c1f90cc0ead2666c91fa3e64162f76fa1b3483f5825339b9f9.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/ywzPcsYIdZ5bOupWECxvCgLem?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ywzPcsYIdZ5bOupWECxvCgLem.png" width="836"/>
</a>
</details>

##### use-seed-phrase - Use seed phrase to create an implicit account

Данная команда с помощью мнемонической фразы генерирует ключи доступа и сохраняет их в файле с именем _implicit-account-id_.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account \
    fund-later \
    use-seed-phrase 'start vote foot cereal link cabin fantasy universe hero drama bird fiction' \
        --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' \
    save-to-folder /Users/frovolod/.near-credentials/implicit
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO The file "/Users/frovolod/.near-credentials/implicit/eca9e1a6e0fa9a6af6d046bcffa6508f90f98e646836647ecd883d1d2b1989e5.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/g8IGfYHTeitrtGwcr3deCosG9?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/g8IGfYHTeitrtGwcr3deCosG9.png" width="836"/>
</a>
</details>

#### update-social-profile - Update NEAR Social profile

- [json-args](#json-args---Valid-JSON-arguments-eg-token_id-42)
- base64-args
- [file-args](#file-args---Read-from-file-eg-reusable-JSON-or-binary-data)
- [manually](#manually---Interactive-input-of-arguments)

##### json-args - Valid JSON arguments (e.g. {"token_id": "42"})

Для изменения профиля аккаунта на контракте с использованием аргументов в формате JSON необходимо ввести в командной строке терминала:

```txt
near account \
    update-social-profile fro_volod.testnet \
    json-args '{"name":"frovolod","image":{"ipfs_cid":"bafkreifdzusz6hp3j4njdtqqxr3tlvx4agedgh7znyac4wbuiao3gtppde"},"linktree":{"github":"FroVolod","telegram":"frovolod"},"tags": {"rust":"","near":"","developer":""}}' \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Profile for fro_volod.testnet updated successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/o1syzzHQ6NDAXp2HOKTw2vA7V?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/o1syzzHQ6NDAXp2HOKTw2vA7V.png" width="836"/>
</a>
</details>

##### file-args - Read from file (e.g. reusable JSON or binary data)

Для изменения профиля аккаунта на контракте с использованием подготовленного файла необходимо ввести в командной строке терминала:

```txt
near account \
    update-social-profile fro_volod.testnet \
    file-args profile.txt \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Profile for fro_volod.testnet updated successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/Uc28SNzhjRE2qJAdo6DSuuia4?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/Uc28SNzhjRE2qJAdo6DSuuia4.png" width="836"/>
</a>
</details>

##### manually - Interactive input of arguments

Для изменения профиля аккаунта на контракте с использованием интерактивного режима необходимо воспользоваться диалоговыми подсказками либо ввести в командной строке терминала:

```txt
near account \
    update-social-profile fro_volod.testnet \
    manually \
        --name fro_volod.testnet \
        --image-ipfs-cid bafkreifdzusz6hp3j4njdtqqxr3tlvx4agedgh7znyac4wbuiao3gtppde \
        --description 'This is my profile' \
        --github FroVolod \
        --website https://auto-rti.com/ \
        --tags dev,rust \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Profile for fro_volod.testnet updated successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/sJxaZKOkjGu75yvMGOqkQxi34?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/sJxaZKOkjGu75yvMGOqkQxi34.png" width="836"/>
</a>
</details>

#### delete-account - Delete an account

Данная команда предназначена для удаления текущего аккаунта. Важно помнить, что все средства удаляемого аккаунта перейдут на счет "_beneficiary_".
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    delete-account test_fro.testnet \
    beneficiary volodymyr.testnet \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [test_fro.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    Account <test_fro.testnet> has been successfully deleted.

 |    Gas burned: 0.512 Tgas
 |    Transaction fee: 0.0000511097 NEAR (approximately $0.00017019 USD, using $3.33 USD/NEAR exchange rate)
 |    Transaction ID: GZjvB6sDetrShK6bDHpZTgeuSRuwEgP1vfDzsGrsfo1o
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsGZjvB6sDetrShK6bDHpZTgeuSRuwEgP1vfDzsGrsfo1o
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/pnDBuxBmhq510wgFH894hUcwP?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/pnDBuxBmhq510wgFH894hUcwP.png" width="836"/>
</a>
</details>

#### list-keys - View a list of access keys of an account

Просмотр ключей доступа аккаунта возможен на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для просмотра списка ключей доступа необходимо ввести в командной строке терминала:
```txt
near account \
    list-keys fro_volod.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
+----+------------------------------------------------------+-----------------+------------------------------------------------------------------------------------------------+
| #  | Public Key                                           | Nonce           | Permissions                                                                                    |
+----+------------------------------------------------------+-----------------+------------------------------------------------------------------------------------------------+
| 1  | ed25519:1TprKa4burMqDMjDHyBSUaFQQczF7NamhxTx2yEXe9P  | 116133598000035 | full access                                                                                    |
| 2  | ed25519:51oCqnMN2qcYsG7uVREEeJsodnodqcWqnLonLuHynjs  | 102558993000058 | full access                                                                                    |
| 3  | ed25519:9Wmqx7NmztxtBeMAmwe6V4PrhedEo8Wh7EmjqzpFeGU  | 188577382000028 | do any function calls on v1.social08.testnet with an allowance of 0.240 NEAR                   |
| 4  | ed25519:PgDd7jVtz9oHMrVPJXQKfVTVKgeYN48Xgcsad6pvvB7  | 105045425000000 | full access                                                                                    |
| 5  | ed25519:RtG1Pg8ZeuTxRYqtc3fmhJWJAMEDZqtQAfDZHjKChh2  | 115787893000227 | full access                                                                                    |
| 6  | ed25519:eXeEYjNKj6qNsy2HenhFPQ2DuN6JUpDjffmmyEr8WFj  | 101440281000000 | full access                                                                                    |
| 7  | ed25519:27R66L6yevyHbsk4fESZDC8QUQBwCdx6vvkk1uQmG7NY | 97890993000000  | only do ["set_a", "set_b"] function calls on meta.pool.testnet with an allowance of 0.100 NEAR |
| 8  | ed25519:2PFtFn3Pd61bHRWf2jwkF53pdCQTWAZiMfe6bF8Wx1k2 | 166608694000003 | do any function calls on v1.social08.testnet with an allowance of 0.249 NEAR                   |
| 9  | ed25519:2QFAeUutKUDpmgKDyHXm7Wcz1uhjxk92fK6zY2dB7FCD | 97492076000000  | do any function calls on v2.ref-farming.testnet with an allowance of 0.250 NEAR                |
| 10 | ed25519:2SBFq3hdLXTCTEfFL6Y5Df7vUSxMsHbtyJLeLbNvyu8o | 102449374000004 | full access                                                                                    |
| 11 | ed25519:2igdi4TVH8saGLonAhdBdbPGpzQNpePktpzfjgX9dzPb | 140356139000001 | do any function calls on v1.social08.testnet with an allowance of 0.250 NEAR                   |
| 12 | ed25519:39rNXzNAHG5UQHs481yr7Kwf5ay5mTqLeC5Ru9Guz1TC | 126060275000006 | do any function calls on v1.social08.testnet with an allowance of 0.244 NEAR                   |
| 13 | ed25519:3Liiip4dG9ixaHiqHwqg9gZ8u9LQVhm89ys9ZKpSHjtD | 188577281000000 | do any function calls on v1.social08.testnet with an allowance of 0.250 NEAR                   |
| 14 | ed25519:3MEZZ2m2VL1XLYu2HNpHREQWNBGatE64XjdKeHAQWBuV | 101494304000024 | full access                                                                                    |
+----+------------------------------------------------------+-----------------+------------------------------------------------------------------------------------------------+
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/wJBFTtuVy76Z7XI8EF3iCnl3b?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/wJBFTtuVy76Z7XI8EF3iCnl3b.png" width="836"/>
</a>
</details>

#### get-public-key - Get the public key to your account

- [from-ledger](#from-ledger---Get-the-public-key-stored-on-your-Ledger-Nano-device)
- [from-seed-phrase](#from-seed-phrase---Get-the-public-key-with-the-seed-phrase)
- [from-plaintext-private-key](#from-plaintext-private-key---Get-the-public-key-from-the-plaintext-private-key)
- [from-keychain](#from-keychain---Get-the-public-key-stored-in-a-secure-keychain)
- [from-legacy-keychain](#from-legacy-keychain---Get-the-public-key-stored-in-the-legacy-keychain-compatible-with-the-old-near-CLI)

#### from-ledger - Get the public key stored on your Ledger Nano device

Для получения публичного ключа из Ledger необходимо ввести в командной строке терминала (обратите внимание, что в качестве путей `from-ledger` поддерживаются только пути, состоящие ровно из 5 компонентов, то есть "m/44'/397'/0'/0'/10'" будет работать, а "m/44'/397'/0'" - нет, в отличие, например, от пути `from-seed-phrase`):
```txt
near account \
    get-public-key \
    from-ledger --seed-phrase-hd-path "m/44'/397'/0'/0'/1'"
```

<details><summary><i>Результат выполнения команды будет примерно таким</i></summary>

```txt
Public key (printed to stdout): 
ed25519:FsRjjvkQZbwcBooXyuz4WMxXtxEKLJVJ6nc3CnaurdRr
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/jlZk2P1wCLJG6tpR8YeNONNBr?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/jlZk2P1wCLJG6tpR8YeNONNBr.png" width="836"/>
</a>
</details>

#### from-seed-phrase - Get the public key with the seed phrase

Для получения публичного ключа из сид-фразы необходимо ввести в командной строке терминала:
```txt
near account \
    get-public-key \
    from-seed-phrase 'trigger arrow grunt vendor crane safe reflect please sponsor verify club shiver' \
        --seed-phrase-hd-path "m/44'/397'/0'"
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Public key (printed to stdout): 
ed25519:3fm1ctizEANiJG2CgJXx41e18BjtNTAnB4hfYSMjd4Fh
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/85kbNU8gg0i0LU1Cm9n9umM78?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/85kbNU8gg0i0LU1Cm9n9umM78.png" width="836"/>
</a>
</details>

#### from-plaintext-private-key - Get the public key from the plaintext private key

Для получения публичного ключа из секретного ключа необходимо ввести в командной строке терминала:
```txt
near account \
    get-public-key \
    from-plaintext-private-key ed25519:3AoMxLat91aAdkh4vyq7MgbKepYhSiC5WzknLFbiXUKfsoCXXeuN9W6R4EpFd3TLvBms7gbafupvtvQJmBt7W24f
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Public key (printed to stdout): 
ed25519:3fm1ctizEANiJG2CgJXx41e18BjtNTAnB4hfYSMjd4Fh
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/XV4rJx6VVxQaKDVoDz2l6Vudt?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/XV4rJx6VVxQaKDVoDz2l6Vudt.png" width="836"/>
</a>
</details>

#### from-keychain - Get the public key stored in a secure keychain

Для получения публичного ключа из Keychain необходимо ввести в командной строке терминала (обратите внимание, что команда может блокировать разблокировку `Keychain` через gui-widget за пределами ввода/вывода cli):
```txt
near account \
    get-public-key \
    from-keychain volodymyr.testnet \
    network-config testnet
```

<details><summary><i>Результат выполнения этой команды для другой учетной записи будет аналогичен следующему:</i></summary>

```txt
Public key (printed to stdout): 
ed25519:CuXJ7CSpDdHvoKMwA4whFnaUwQuWGNCLu7APHKiQb4az
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/9Ct7taSDiQa6BsW5z7iZtXXlA?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9Ct7taSDiQa6BsW5z7iZtXXlA.png" width="836"/>
</a>
</details>

#### from-legacy-keychain - Get the public key stored in the legacy keychain (compatible with the [old near CLI](https://github.com/near/near-cli))

Для получения публичного ключа из хранилища необходимо ввести в командной строке терминала (Каталог с ключами доступа определен в [конфигурационном файле](#config---manage-connections-in-a-configuration-file).
        Ключи доступа должны находиться в файле _публичный-ключ.json_, расположенном в _/Users/user/.near-credentials/имя-сети/имя-пользователя/_):
```txt
near account \
    get-public-key \
    from-legacy-keychain volodymyr.testnet \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Public key (printed to stdout): 
ed25519:CuXJ7CSpDdHvoKMwA4whFnaUwQuWGNCLu7APHKiQb4az
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/w1odwOVXSv1fwOl4BKyoxJNgv?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/w1odwOVXSv1fwOl4BKyoxJNgv.png" width="836"/>
</a>
</details>

#### add-key - Add an access key to an account

Выполним команду добавления новой пары ключей доступа аккаунту с такими условиями:
  - публичный ключ доступа будет введен вручную
  - ключи будут иметь полный доступ
  - транзакция будет подписана автоматически (при наличии файла с ключами доступа)

Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    add-key fro_volod.testnet \
    grant-full-access \
    use-manually-provided-public-key ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [fro_volod.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    Added access key = ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 to fro_volod.testnet.

 |    Gas burned: 0.420 Tgas
 |    Transaction fee: 0.000041964925 NEAR (approximately $0.00013135 USD, using $3.13 USD/NEAR exchange rate)
 |    Transaction ID: 2UNZbYQN6HvzhkT65igKcX3V7U972aUTREahoH8qLXnP
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions2UNZbYQN6HvzhkT65igKcX3V7U972aUTREahoH8qLXnP
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/WJgcapQLRFjFl8WK5EP7ag4GT?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/WJgcapQLRFjFl8WK5EP7ag4GT.png" width="836"/>
</a>
</details>

Изменим наши парамерты для добавления кючей доступа:
  - ключи будут сгенерированы автоматически
  - будут иметь функциональный доступ
  - транзакция будет подписана вручную

Для этого введем следующую команду:
```txt
near account \
    add-key fro_volod.testnet \
    grant-function-call-access \
        --allowance '0.1 NEAR' \
        --contract-account-id meta.pool.testnet \
        --function-names 'set_a, set_b' \
    autogenerate-new-keypair \
    save-to-keychain \
    network-config testnet \
    sign-with-plaintext-private-key \
        --signer-public-key ed25519:1TprKa4burMqDMjDHyBSUaFQQczF7NamhxTx2yEXe9P \
        --signer-private-key ed25519:1aXaNaPNxU6Nwb4R1FxP9FzqFqhXwsx3nDS8PWv2jLxcX2ABEbKiGCPFKwEqQYzULWqiXLZDQX8oZYrhSLnDXFf \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [fro_volod.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    Added access key = ed25519:GmYFKxZ85UETqmnwCpqEHiy5ZW2YNCj75hM2rvADyXW9 to fro_volod.testnet.

 |    Gas burned: 0.421 Tgas
 |    Transaction fee: 0.0000420600457944 NEAR (approximately $0.00013921 USD, using $3.31 USD/NEAR exchange rate)
 |    Transaction ID: HTpGEukqkBTmYowVgyWAfLFVXBFUUZr9bgdGq865H63X
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsHTpGEukqkBTmYowVgyWAfLFVXBFUUZr9bgdGq865H63X

 INFO 
 |    The data for the access key is saved in the keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/ob8maBfWAkmzcdkDkyiFj3NUN?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ob8maBfWAkmzcdkDkyiFj3NUN.png" width="836"/>
</a>
</details>

#### delete-keys - Delete access keys from an account

Для удаления ключей доступа необходимо ввести в командной строке терминала:
```txt
near account \
    delete-keys fro_volod.testnet \
    public-keys ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [fro_volod.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    Access key <ed25519:1TprKa4burMqDMjDHyBSUaFQQczF7NamhxTx2yEXe9P> for account <fro_volod.testnet> has been successfully deleted.

 |    Gas burned: 0.407 Tgas
 |    Transaction fee: 0.000040601225 NEAR (approximately $0.00013357 USD, using $3.29 USD/NEAR exchange rate)
 |    Transaction ID: EnEZCBbpbYnxw1owzdezt78VDffBSV947zAruS9JnYx7
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsEnEZCBbpbYnxw1owzdezt78VDffBSV947zAruS9JnYx7
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/0YaxX4K0CbV5E4Ub4SycxwDoq?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/0YaxX4K0CbV5E4Ub4SycxwDoq.png" width="836"/>
</a>
</details>

#### manage-storage-deposit - Storage management: deposit, withdrawal, balance review

- [view-balance](#view-balance---View-storage-balance-for-an-account)
- [deposit](#deposit---Make-a-storage-deposit-for-the-account)
- [withdraw](#withdraw---Withdraw-a-deposit-from-storage-for-an-account-ID)

##### view-balance - View storage balance for an account

Для просмотра баланса аккаунта на контракте на последнем блоке необходимо ввести в командной строке терминала:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    view-balance volodymyr.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
storage balance for <volodymyr.testnet>:
 available:        1.6 MB   (15.878059999854543210876557 NEAR [  15878059999854543210876557 yoctoNEAR])
 total:            1.6 MB   (16.238949999854543210876557 NEAR [  16238949999854543210876557 yoctoNEAR])
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/mxCOOQk8xRLvY4mIhDsrapwmG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/mxCOOQk8xRLvY4mIhDsrapwmG.png" width="836"/>
</a>
</details>

##### deposit - Make a storage deposit for the account

Для пополнения баланса аккаунта на контракте необходимо ввести в командной строке терминала:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    deposit volodymyr.testnet '1 NEAR' \
    sign-as fro_volod.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [v1.social08.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    {
 |      "available": "18240389999854543210876607",
 |      "total": "28338949999854543210876607"
 |    }
 |    ------------------------------------

 |    The "storage_deposit" call to <v1.social08.testnet> on behalf of <fro_volod.testnet> succeeded.

 |    Gas burned: 2.7 Tgas
 |    Transaction fee: 0.0002640055798606 NEAR (approximately $0.00071017 USD, using $2.69 USD/NEAR exchange rate)
 |    Transaction ID: 4hdrNYjpTMD4crncQ2dSkvwTu4Nn5gCoxx73KcjX6mSQ
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions4hdrNYjpTMD4crncQ2dSkvwTu4Nn5gCoxx73KcjX6mSQ

 INFO 
 |    <fro_volod.testnet> has successfully added a deposit of 1 NEAR to <volodymyr.testnet> on contract <v1.social08.testnet>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/DlT4UZoCGaSJRJG90gAWuEYau?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/DlT4UZoCGaSJRJG90gAWuEYau.png" width="836"/>
</a>
</details>

##### withdraw - Withdraw a deposit from storage for an account ID

Для вывода средств с баланса аккаунта на контракте необходимо ввести в командной строке терминала:

```txt
near account \
    manage-storage-deposit v1.social08.testnet \
    withdraw '0.5 NEAR' \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [v1.social08.testnet]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    {
 |      "available": "17540389999854543210876607",
 |      "total": "27638949999854543210876607"
 |    }
 |    ------------------------------------

 |    The "storage_withdraw" call to <v1.social08.testnet> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 3.6 Tgas
 |    Transaction fee: 0.000334496827071 NEAR (approximately $0.00090648 USD, using $2.71 USD/NEAR exchange rate)
 |    Transaction ID: SBmgKggqKy7NuhK51Ug2JRYYeEwT453uqQe5ntHDwUJ
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsSBmgKggqKy7NuhK51Ug2JRYYeEwT453uqQe5ntHDwUJ

 INFO 
 |    <volodymyr.testnet> has successfully withdraw 0.5 NEAR from <v1.social08.testnet>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/YKOPSaGn7WGJl4tBBTjk2X4Qf?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/YKOPSaGn7WGJl4tBBTjk2X4Qf.png" width="836"/>
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

Данная команда служит для перевода средств NEAR токенах между аккаунтами. Обратите внимание, что количество пересылаемых токенов указывается совместно с размерной единицей (это NEAR либо yoctoNEAR).
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    send-near volodymyr.testnet 0.1NEAR \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [volodymyr.testnet]:   No logs
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    <fro_volod.testnet> has transferred 0.1 NEAR to <volodymyr.testnet> successfully.

 |    Gas burned: 0.447 Tgas
 |    Transaction fee: 0.0000446365125 NEAR (approximately $0.00014506 USD, using $3.25 USD/NEAR exchange rate)
 |    Transaction ID: FjU9rvNvaUUwKgFnH7UUmSEYuB3LBKBgY8QPwnfSgwVH
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsFjU9rvNvaUUwKgFnH7UUmSEYuB3LBKBgY8QPwnfSgwVH
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/6ZwQzkCc1y6QlG1u2gp7taQuz?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/6ZwQzkCc1y6QlG1u2gp7taQuz.png" width="836"/>
</a>
</details>

#### send-ft - The transfer is carried out in FT tokens

Данная команда служит для перевода средств в FT токенах между аккаунтами. Обратите внимание, что количество пересылаемых токенов указывается с названием валюты токена.  
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    send-ft usdn.testnet volodymyr.testnet '10 usn' memo Memo \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [usdn.testnet]:
 |      EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_transfer","data":[{"old_owner_id":"fro_volod.testnet","new_owner_id":"volodymyr.testnet","amount":"10000000000000000000","memo":"Memo"}]}
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "ft_transfer" call to <usdn.testnet> on behalf of <fro_volod.testnet> succeeded.

 |    Gas burned: 3.3 Tgas
 |    Transaction fee: 0.0003208356830642 NEAR (approximately $0.00104913 USD, using $3.27 USD/NEAR exchange rate)
 |    Transaction ID: 53divo1wG2Qbod9NpHrtb2jLhoMYjr79nx4BWgpXToBV
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions53divo1wG2Qbod9NpHrtb2jLhoMYjr79nx4BWgpXToBV

 INFO 
 |    <fro_volod.testnet> has successfully transferred 10 USN (FT-contract: usdn.testnet) to <volodymyr.testnet>.
 |    Remaining balance: 19633813.798969034783801448 USN
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/8GTPGhYidBtk5PUfXTphoMykx?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/8GTPGhYidBtk5PUfXTphoMykx.png" width="836"/>
</a>
</details>

В случае, если Вы желаете перевести все токены, имеющиеся на Вашем аккаунте, вместо точного количества токенов напечатайте "all".  
Обратите внимание, что параметр "prepaid-gas" по умолчанию установлен "100.0 Tgas", а параметр "attached-deposit" - "1 yoctoNEAR", но их можно изменить.
```txt
near tokens \
    volodymyr.testnet \
    send-ft usdn.testnet fro_volod.testnet all memo '' \
        --prepaid-gas '300.0 Tgas' \
        --attached-deposit '1 yoctoNEAR' \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [usdn.testnet]:
 |      EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_transfer","data":[{"old_owner_id":"volodymyr.testnet","new_owner_id":"fro_volod.testnet","amount":"20000000000000000000"}]}
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "ft_transfer" call to <usdn.testnet> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 3.2 Tgas
 |    Transaction fee: 0.0003185247202124 NEAR (approximately $0.00104157 USD, using $3.27 USD/NEAR exchange rate)
 |    Transaction ID: 3ThPcpCHV7xAjpd6MXkVtcG4E7RYN8XLdsopu17dKtzy
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions3ThPcpCHV7xAjpd6MXkVtcG4E7RYN8XLdsopu17dKtzy

 INFO 
 |    <volodymyr.testnet> has successfully transferred 20 USN (FT-contract: usdn.testnet) to <fro_volod.testnet>.
 |    Remaining balance: 0 USN
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/9QU5VNUACLx39P8IuMcpiRgq3?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9QU5VNUACLx39P8IuMcpiRgq3.png" width="836"/>
</a>
</details>

#### send-nft - The transfer is carried out in NFT tokens

Данная команда служит для перевода средств в NFT токенах между аккаунтами.  
Обратите внимание, что параметр "prepaid-gas" по умолчанию установлен "100.0 Tgas", а параметр "attached-deposit" - "1 yoctoNEAR", но их можно изменить.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    send-nft paras-token-v2.testnet volodymyr.testnet 1604:4 \
        --prepaid-gas '300.0 Tgas' \
        --attached-deposit '1 yoctoNEAR' \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [paras-token-v2.testnet]:
 |      Transfer 1604:4 from fro_volod.testnet to volodymyr.testnet
 |      EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_transfer","data":[{"old_owner_id":"fro_volod.testnet","new_owner_id":"volodymyr.testnet","token_ids":["1604:4"]}]}
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "nft_transfer" call to <paras-token-v2.testnet> on behalf of <fro_volod.testnet> succeeded.

 |    Gas burned: 7.0 Tgas
 |    Transaction fee: 0.0006925168715809 NEAR (approximately $0.00221605 USD, using $3.20 USD/NEAR exchange rate)
 |    Transaction ID: 5hU6kfPak5pbZjC7ovs1jSiaoFHtKYWr5KUnuWb2fXc2
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions5hU6kfPak5pbZjC7ovs1jSiaoFHtKYWr5KUnuWb2fXc2

 INFO 
 |    <fro_volod.testnet> has successfully transferred NFT token_id="1604:4" to <volodymyr.testnet> on contract <paras-token-v2.testnet>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/KE8sxQqiF56YOunQjN0v3ShuF?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KE8sxQqiF56YOunQjN0v3ShuF.png" width="836"/>
</a>
</details>

#### view-near-balance - View the balance of Near tokens

Просмотр баланса аккаунта возможен на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для просмотра средств в NEAR токенах на счету аккаунта необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    view-near-balance \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    fro_volod.testnet account has 3071.44 NEAR available for transfer (the total balance is 3074.41 NEAR, but 2.97 NEAR is locked for storage)
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/UnX0qgb8zN7nRkb7dUk9vP3kL?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/UnX0qgb8zN7nRkb7dUk9vP3kL.png" width="836"/>
</a>
</details>

#### view-ft-balance - View the balance of FT tokens

Просмотр баланса аккаунта возможен на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для просмотра средств в FT токенах на счету аккаунта необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    view-ft-balance usdn.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO <fro_volod.testnet> account has 19633875.798969034783801448 USN  (FT-contract: usdn.testnet)
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/H2G535BxCM4qB6s3tPmusfHb0?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/H2G535BxCM4qB6s3tPmusfHb0.png" width="836"/>
</a>
</details>

#### view-nft-assets - View the balance of NFT tokens

Просмотр баланса аккаунта возможен на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для просмотра средств в NFT токенах на счету аккаунта необходимо ввести в командной строке терминала:
```txt
near tokens \
    fro_volod.testnet \
    view-nft-assets paras-token-v2.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
fro_volod.testnet account has NFT tokens:
 INFO fro_volod.testnet account has NFT tokens:
 |    [
 |      {
 |        "approved_account_ids": {},
 |        "metadata": {
 |          "copies": 100,
 |          "description": null,
 |          "expires_at": null,
 |          "extra": null,
 |          "issued_at": "1657613801537412611",
 |          "media": "bafybeib65t37t2tagukok4m7f5rldfirzb5ykvdq3yqbwnbcrtllpggg6u",
 |          "media_hash": null,
 |          "reference": "bafkreidmbv4j2qylxc2mngsup7cxakw7gwyd7lu2zycznrdtqw4kc52cwu",
 |          "reference_hash": null,
 |          "starts_at": null,
 |          "title": "Apollo42 #01 #4",
 |          "updated_at": null
 |        },
 |        "owner_id": "fro_volod.testnet",
 |        "token_id": "1604:4"
 |      }
 |    ]
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/bRStRj3bg1gT9YAwFxeScFcai?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bRStRj3bg1gT9YAwFxeScFcai.png" width="836"/>
</a>
</details>

### staking - Manage staking: view, add and withdraw stake

- [validator-list](#validator-list---View-the-list-of-validators-to-delegate)
- [delegation](#delegation---Delegation-management)

#### validator-list - View the list of validators to delegate

Для просмотра списка валидаторов необходимо ввести в командной строке терминала:
```txt
near staking \
    validator-list \
    network-config mainnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
+-----+----------------------------------------------+----------+------------+----------------------------------------+
| #   | Validator Id                                 | Fee      | Delegators | Stake                                  |
+-----+----------------------------------------------+----------+------------+----------------------------------------+
| 1   | staked.poolv1.near                           |     10 % |     3207   | 44135674.18356215181482959363448 NEAR  |
| 2   | figment.poolv1.near                          |     10 % |     1911   | 43158696.364374348313201031661037 NEAR |
| 3   | astro-stakers.poolv1.near                    |      1 % |    11528   | 26760042.204197815051321354819805 NEAR |
| 4   | bzam6yjpnfnxsdmjf6pw.poolv1.near             |    100 % |      772   | 23347900.996610021010359525969384 NEAR |
| 5   | zavodil.poolv1.near                          |      1 % |     7116   | 20700903.223980192761611953425855 NEAR |
| 6   | binancenode1.poolv1.near                     |      5 % |     1250   | 14209385.916611355199355410152982 NEAR |
| 7   | staking_yes_protocol1.poolv1.near            |    100 % |       65   | 13590245.381034035922399111793022 NEAR |
| 8   | pinnacle1.poolv1.near                        |    100 % |        4   | 13509874.537453205747773186007329 NEAR |
| 9   | priory.poolv1.near                           |    100 % |       15   | 12727257.514716521676379711750814 NEAR |
| 10  | stake1.poolv1.near                           |      3 % |      754   | 12449700.095021989100340879377004 NEAR |
| 11  | mockingbird.poolv1.near                      |    100 % |       28   | 11501759.018634341466180769487983 NEAR |
| 12  | dqw9k3e4422cxt92masmy.poolv1.near            |    100 % |       36   | 11122519.385245577197951932017032 NEAR |
| 13  | flipside.pool.near                           |    100 % |        9   | 11087540.718366137730589600283212 NEAR |
| 14  | sweat_validator.poolv1.near                  |    100 % |      112   | 10900424.272450229667472212076621 NEAR |
| 15  | epic.poolv1.near                             |      1 % |     5363   | 10769900.629411406438519703653828 NEAR |
| 16  | future_is_near.poolv1.near                   |      9 % |      355   | 10243082.132364573976720438585765 NEAR |
| 17  | cosmose.poolv1.near                          |    100 % |       10   | 10064982.806109296980776431396738 NEAR |
| 18  | aurora.pool.near                             |     99 % |     3301   | 9298278.181302142009939675438401 NEAR  |
...
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/IYG8qgo3bdXHrgnyJL443gw6L?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/IYG8qgo3bdXHrgnyJL443gw6L.png" width="836"/>
</a>
</details>

#### delegation - Delegation management

- [view-balance](#view-balance---View-the-delegated-stake-balance-for-a-given-account)
- [deposit-and-stake](#deposit-and-stake---Delegate-NEAR-tokens-to-a-validators-staking-pool)
- [stake](#stake---Delegate-a-certain-amount-of-previously-deposited-or-unstaked-NEAR-tokens-to-a-validators-staking-pool)
- [stake-all](#stake-all---Delegate-all-previously-deposited-or-unstaked-NEAR-tokens-to-a-validators-staking-pool)
- [unstake](#unstake---Unstake-a-certain-amount-of-delegated-NEAR-tokens-from-a-validators-staking-pool)
- [unstake-all](#unstake-all---Unstake-all-delegated-NEAR-tokens-from-a-validators-staking-pool)
- [withdraw](#withdraw---Withdraw-a-certain-amount-of-unstaked-NEAR-tokens-from-a-validators-staking-pool)
- [withdraw-all](#withdraw-all---Withdraw-all-unstaked-NEAR-tokens-from-a-validators-staking-pool)

##### view-balance - View the delegated stake balance for a given account

Для просмотра порученного баланса аккаунта в общий фонд валидатора необходимо ввести в командной строке терминала:

```txt
near staking \
    delegation volodymyr.testnet \
    view-balance aurora.pool.f863973.m0 \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Delegated stake balance with validator <aurora.pool.f863973.m0> by <volodymyr.testnet>:
 |          Staked balance:                                 10.07 NEAR
 |          Unstaked balance:                              139.98 NEAR (available for withdrawal)
 |          Total balance:                                 150.04 NEAR
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/TmMzGE4lfW4PZONfbfF57IRwt?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/TmMzGE4lfW4PZONfbfF57IRwt.png" width="836"/>
</a>
</details>

##### deposit-and-stake - Delegate NEAR tokens to a validator's staking pool

Чтобы поручить ваши NEAR токены в общий фонд одного из валидаторов и получения наград, передайте ваши NEAR токены и переведите их в ставку валидатора, используя следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, указать желаемое количество NEAR токенов, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    deposit-and-stake '15 NEAR' aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [aurora.pool.f863973.m0]:
 |      @volodymyr.testnet deposited 15000000000000000000000000. New unstaked balance is 139970879972821537334942845
 |      @volodymyr.testnet staking 14999999999999999999999996. Received 3440175881468611169391603 new staking shares. Total 124970879972821537334942849 unstaked balance and 22948853294897913527674841 staking shares
 |      Contract total staked balance is 18048777328345645362302984477380. Total number of shares 4139397896998144363779930177559
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "deposit_and_stake" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 6.1 Tgas
 |    Transaction fee: 0.000565088651184 NEAR (approximately $0.00184218 USD, using $3.26 USD/NEAR exchange rate)
 |    Transaction ID: 5mqV2dcZSQZz1RvT9kbKgS68A62sxYCQUyPkfGQ7qsvw
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions5mqV2dcZSQZz1RvT9kbKgS68A62sxYCQUyPkfGQ7qsvw

 INFO 
 |    <volodymyr.testnet> has successfully delegated 15 NEAR to stake with <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/BoKVychKEmbazAeo6jWDbm4KL?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/BoKVychKEmbazAeo6jWDbm4KL.png" width="836"/>
</a>
</details>

##### stake - Delegate a certain amount of previously deposited or unstaked NEAR tokens to a validator's staking pool

Чтобы поручить ваши NEAR токены в общий фонд одного из валидаторов и получения наград, переведите раннее переданные NEAR токены в ставку валидатора, используя следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, указать желаемое количество NEAR токенов, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    stake '5 NEAR' aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [aurora.pool.f863973.m0]:
 |      @volodymyr.testnet staking 4999999999999999999999998. Received 1146725293822870389797201 new staking shares. Total 124970879972821537334942845 unstaked balance and 19508677413429302358283238 staking shares
 |      Contract total staked balance is 18048762328345645362302984477383. Total number of shares 4139394456822262895168760785956
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "stake" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 5.8 Tgas
 |    Transaction fee: 0.0005278299014306 NEAR (approximately $0.00172072 USD, using $3.26 USD/NEAR exchange rate)
 |    Transaction ID: Cv6VTBzU5v4gjmsGAhTFuY2taKL4RmZZoSJfFkV81Fbt
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsCv6VTBzU5v4gjmsGAhTFuY2taKL4RmZZoSJfFkV81Fbt

 INFO 
 |    <volodymyr.testnet> has successfully delegated 5 NEAR to stake with <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/0yWzlHfbiB0FvX0k4PJuQndyu?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/0yWzlHfbiB0FvX0k4PJuQndyu.png" width="836"/>
</a>
</details>

##### stake-all - Delegate all previously deposited or unstaked NEAR tokens to a validator's staking pool

Чтобы поручить ваши NEAR токены в общий фонд одного из валидаторов и получения наград, переведите все раннее переданные NEAR токены в ставку валидатора, используя следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, а также выбрать идентификатор аккаунта валидатора):
```txt
near staking \
    delegation volodymyr.testnet \
    stake-all aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [aurora.pool.f863973.m0]:
 |      @volodymyr.testnet staking 124970879972821537334942846. Received 28661453811227289230731537 new staking shares. Total 3 unstaked balance and 51610307106125202758406378 staking shares
 |      Contract total staked balance is 18048902299225618183840319420227. Total number of shares 4139426558451955591069160909096
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "stake_all" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 5.9 Tgas
 |    Transaction fee: 0.0005400473556783 NEAR (approximately $0.00174975 USD, using $3.24 USD/NEAR exchange rate)
 |    Transaction ID: BHqnut5dFr9H76K31VqHHw5zgDVNhp3TbjxcmWG87Mg7
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsBHqnut5dFr9H76K31VqHHw5zgDVNhp3TbjxcmWG87Mg7

 INFO 
 |    <volodymyr.testnet> has successfully delegated to stake with <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/gHLsrArPXQZI6cS5a9L0GGIRu?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/gHLsrArPXQZI6cS5a9L0GGIRu.png" width="836"/>
</a>
</details>

##### unstake - Unstake a certain amount of delegated NEAR tokens from a validator's staking pool

Чтобы отменить определённую часть ставки, совершённую через общий фонд валидатора, используйте следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, указать желаемое количество NEAR токенов, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    unstake '7 NEAR' aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [aurora.pool.f863973.m0]:
 |      @volodymyr.testnet unstaking 7000000000000000000000002. Spent 1605415411352018545716082 staking shares. Total 14000000000000000000000005 unstaked balance and 48399476283421165666974215 staking shares
 |      Contract total staked balance is 18048888299225618183840319420230. Total number of shares 4139423347621132887032069476933
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [aurora.pool.f863973.m0]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "unstake" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 5.8 Tgas
 |    Transaction fee: 0.0005277130544669 NEAR (approximately $0.00173089 USD, using $3.28 USD/NEAR exchange rate)
 |    Transaction ID: 2CaQzKqsLiVLc9xSKcZAkL8o9ypPgvYrNDdkvGY7AmU9
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions2CaQzKqsLiVLc9xSKcZAkL8o9ypPgvYrNDdkvGY7AmU9

 INFO 
 |    <volodymyr.testnet> has successfully unstaked 7 NEAR from <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/KDoVnegViGZsLudqI17kTAqn7?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KDoVnegViGZsLudqI17kTAqn7.png" width="836"/>
</a>
</details>

##### unstake-all - Unstake all delegated NEAR tokens from a validator's staking pool

Чтобы отменить всю ставку, совершённую через общий фонд валидатора, используйте следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    unstake-all aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO                                                                                                                                                                         
 |    --- Logs ---------------------------                                                                                                                                    
 |    Logs [aurora.pool.f863973.m0]:                                                                                                                                          
 |      @volodymyr.testnet unstaking 225033438191942506311861536. Spent 51610307106125202758406379 staking shares. Total 225033438191942506311861540 unstaked balance and 0 staking shares                                                                                                                                                                  
 |      Contract total staked balance is 18048677265787426241334007558698. Total number of shares 4139374948144849465866402502718                                             
 |    Logs [aurora.pool.f863973.m0]:   No logs                                                                                                                                
 |    Logs [volodymyr.testnet]:   No logs                                                                                                                                     
 |    Logs [aurora.pool.f863973.m0]:   No logs                                                                                                                                
 |    Logs [volodymyr.testnet]:   No logs                                                                                                                                     
 |    ------------------------------------                                                                                                                                    
 |    --- Result -------------------------                                                                                                                                    
 |    Empty result                                                                                                                                                            
 |    ------------------------------------                                                                                                                                    
                                                                                                                                                                              
 |    The "unstake_all" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.                                                                          
                                                                                                                                                                              
 |    Gas burned: 5.9 Tgas                                                                                                                                                    
 |    Transaction fee: 0.0005414102448012 NEAR (approximately $0.00177582 USD, using $3.28 USD/NEAR exchange rate)                                                            
 |    Transaction ID: Bp2nPibgyDzKTqbGhqeowhZCgdqAjoGYr3PGnBVLDD9X                                                                                                            
 |    To see the transaction in the transaction explorer, please open this url in your browser:                                                                               
 |    https://explorer.testnet.near.org/transactionsBp2nPibgyDzKTqbGhqeowhZCgdqAjoGYr3PGnBVLDD9X                                                                              
                                                                                                                                                                              
 INFO                                                                                                                                                                         
 |    <volodymyr.testnet> has successfully unstaked the entire available amount from <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/QMUCK5dw9hz91zCQntAqF8JFX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/QMUCK5dw9hz91zCQntAqF8JFX.png" width="836"/>
</a>
</details>

##### withdraw - Withdraw a certain amount of unstaked NEAR tokens from a validator's staking pool

Чтобы получить ваши раннее порученные NEAR токены и награды из общего фонда валидатора после того как ставка была снята и прошло 4 эпохи, используйте следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, указать желаемое количество NEAR токенов, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    withdraw '3 NEAR' aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [aurora.pool.f863973.m0]:
 |      @volodymyr.testnet withdrawing 3000000000000000000000000. New unstaked balance is 229033438191942506311861548
 |    Logs [volodymyr.testnet]:   No logs
 |    Logs [volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "withdraw" call to <aurora.pool.f863973.m0> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 3.3 Tgas
 |    Transaction fee: 0.0002987794103032 NEAR (approximately $0.00089932 USD, using $3.01 USD/NEAR exchange rate)
 |    Transaction ID: 9g59iXm8efEjZ2wmK3KEKTNQFAcqXfnVXUQXJroFnPcD
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions9g59iXm8efEjZ2wmK3KEKTNQFAcqXfnVXUQXJroFnPcD

 INFO 
 |    <volodymyr.testnet> has successfully withdrawn 3 NEAR from <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/kfkWKm88jwOWYnmbJptbEH618?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/kfkWKm88jwOWYnmbJptbEH618.png" width="836"/>
</a>
</details>

##### withdraw-all - Withdraw all unstaked NEAR tokens from a validator's staking pool

Чтобы получить все ваши раннее порученные NEAR токены и награды из общего фонда валидатора после того как ставка была снята и прошло 4 эпохи, используйте следующую командну в терминале (обратите внимание, что вам необходимо использовать свой собственный идентификатор аккаунта, а также выбрать идентификатор аккаунта валидатора):

```txt
near staking \
    delegation volodymyr.testnet \
    withdraw-all aurora.pool.f863973.m0 \
    network-config testnet \
    sign-with-legacy-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Error:
   0: <volodymyr.testnet> can't withdraw tokens in the current epoch.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/5ql7FP93TM2whN2kyVYxBCtYy?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/5ql7FP93TM2whN2kyVYxBCtYy.png" width="836"/>
</a>
</details>

### contract - Manage smart-contracts: deploy code, call functions

- [call-function](#call-function---Execute-function-contract-method)
- [deploy](#deploy---Add-a-new-contract-code)
- [inspect](#inspect---Get-a-list-of-available-function-names)
- [verify](#verify---Verify-the-contract-for-compliance-with-the-program-code)
- [download-abi](#download-abi---Download-contract-ABI)
- [download-wasm](#download-wasm---Download-wasm)
- [view-storage](#view-storage---View-contract-storage-state)

#### call-function - Execute function (contract method)

- [as-read-only](#as-read-only---Calling-a-view-method)
- [as-transaction](#as-transaction---Calling-a-change-method)

##### as-read-only - Calling a view method

Просмотр данных возможен на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near contract \
    call-function \
    as-read-only zavodil.poolv1.near get_accounts \
    json-args '{"from_index": 0, "limit": 3}' \
    network-config mainnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    No logs
 |    ------------------------------------
 INFO 
 |    --- Result -------------------------
 |    [
 |      {
 |        "account_id": "zavodil.near",
 |        "can_withdraw": true,
 |        "staked_balance": "11433121116815084999423646794",
 |        "unstaked_balance": "0"
 |      },
 |      {
 |        "account_id": "dba22fecd3b52fbba153f476dd6ea166b9b1c5f2b73a51461ff738445b195181",
 |        "can_withdraw": true,
 |        "staked_balance": "3331729047758900549893273",
 |        "unstaked_balance": "1"
 |      },
 |      {
 |        "account_id": "gibby49.near",
 |        "can_withdraw": true,
 |        "staked_balance": "1405036979648505277794095",
 |        "unstaked_balance": "1"
 |      }
 |    ]
 |    ------------------------------------
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/O0wuaKKU9aaDLZOQk0R33mYvn?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/O0wuaKKU9aaDLZOQk0R33mYvn.png" width="836"/>
</a>
</details>

##### as-transaction - Calling a change method

Для выполнения этой команды необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [turbo.volodymyr.testnet]:   No logs
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------

 |    The "rate" call to <turbo.volodymyr.testnet> on behalf of <fro_volod.testnet> succeeded.

 |    Gas burned: 2.2 Tgas
 |    Transaction fee: 0.0002154134874181 NEAR (approximately $0.00070440 USD, using $3.27 USD/NEAR exchange rate)
 |    Transaction ID: DVB2RxNJyazoAKxMs7VugWuqiU9ZgkVXvLmM7cxs88jf
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactionsDVB2RxNJyazoAKxMs7VugWuqiU9ZgkVXvLmM7cxs88jf
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/6yYpaiRUa3b80P5ECGeX9SnLy?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/6yYpaiRUa3b80P5ECGeX9SnLy.png" width="836"/>
</a>
</details>

#### deploy - Add a new contract code

Для добавления нового контракта необходимо ввести в командной строке терминала:
```txt
near contract \
    deploy \
    volodymyr.testnet \
    use-file /Users/frovolod/Documents/NEAR/near-cli-rs/counter_volodymyr_testnet.wasm \
    with-init-call increment \
    json-args {} \
    prepaid-gas '100 TGas' \
    attached-deposit '0 NEAR' \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    --- Logs ---------------------------
 |    Logs [volodymyr.testnet]:
 |      Increased number to 43
 |      Make sure you don't overflow, my friend.
 |    ------------------------------------
 |    --- Result -------------------------
 |    Empty result
 |    ------------------------------------
 |    Contract code has been successfully deployed.
 |    The "increment" call to <volodymyr.testnet> on behalf of <volodymyr.testnet> succeeded.

 |    Gas burned: 10.4 Tgas
 |    Transaction fee: 0.0010399389813202 NEAR (approximately $0.00341099 USD, using $3.28 USD/NEAR exchange rate)
 |    Transaction ID: 3kq668vjhE1ZFFSKegNARfjy8ZhCeit8cPvuY8tELSGF
 |    To see the transaction in the transaction explorer, please open this url in your browser:
 |    https://explorer.testnet.near.org/transactions3kq668vjhE1ZFFSKegNARfjy8ZhCeit8cPvuY8tELSGF
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/EP1iriayC6fZdB6ddz82nBc9W?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/EP1iriayC6fZdB6ddz82nBc9W.png" width="836"/>
</a>
</details>

#### inspect - Get a list of available function names

Для просмотра свойств контракта необходимо ввести в командной строке терминала:
```txt
near contract \
    inspect \
    neardevhub-contract.volodymyr.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
---------------------------------------------------------------------------------------------------------
 neardevhub-contract.volodymyr.testnet  At block #192197254 
                                        (9TU8wpXFYYEuAuiR8Ar9dJWHHxJhJhspmMFHsvpDco8Y) 
---------------------------------------------------------------------------------------------------------
 SHA-256 checksum hex                   af9739ffcf42e571b7d512c35c18d4716ac20d20cae8aafdf6d27cb7bc5040ab 
---------------------------------------------------------------------------------------------------------
 Storage used                           895.9 KB (895.2 KB Wasm + 705 B data) 
---------------------------------------------------------------------------------------------------------
 Access keys                            1 full access keys and 0 function-call-only access keys 
---------------------------------------------------------------------------------------------------------
 Contract version                       0.1.0 
---------------------------------------------------------------------------------------------------------
 Contract link                           
---------------------------------------------------------------------------------------------------------
 Supported standards                    nep330 (1.1.0) 
---------------------------------------------------------------------------------------------------------
 NEAR ABI version                       0.4.0 
---------------------------------------------------------------------------------------------------------

 Functions: (hint: you can download full JSON Schema using `download-abi` command)
  
▹▹▸▹▹ Contract inspection ...  
▹▹▹▹▸  ↳  Analysis of contract data ...                                                                                                                                                                                             ... Arguments (JSON Schema):
       {
         "serialization_type": "json",
         "args": [
           {
             "name": "post_id",
             "type_schema": {
               "type": "integer",
               "format": "uint64",
               "minimum": 0.0
             }
           }
         ]
       }
 ... Return Value (JSON Schema):
       No return value
  
 fn add_member(...) -> ... (read-write function - transcation required)  
 ... Arguments (JSON Schema):
       {
         "serialization_type": "json",
         "args": [
           {
             "name": "member",
             "type_schema": {
               "$ref": "#/definitions/Member"
             }
           },
           {
             "name": "metadata",
             "type_schema": {
               "$ref": "#/definitions/VersionedMemberMetadata"
             }
           }
         ]
       }
 ... Return Value (JSON Schema):
       No return value
  
 fn add_post(...) -> ... (read-write function - transcation required) payable  
 ... Arguments (JSON Schema):
       {
         "serialization_type": "json",
         "args": [
           {
             "name": "parent_id",
             "type_schema": {
               "type": [
                 "integer",
                 "null"
               ],
               "format": "uint64",
               "minimum": 0.0
             }
           },
           {
             "name": "body",
             "type_schema": {
               "$ref": "#/definitions/PostBody"
             }
           },
           {
             "name": "labels",
             "type_schema": {
               "type": "array",
               "items": {
                 "type": "string"
               },
               "uniqueItems": true
             }
           }
         ]
       }
 ... Return Value (JSON Schema):
       No return value
... ... ...       
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/FKvZYq7vHu0zwvw3KYSKwQuBB?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/FKvZYq7vHu0zwvw3KYSKwQuBB.png" width="836"/>
</a>
</details>

#### verify - Verify the contract for compliance with the program code

Для проверки контракта на соответствие программному коду необходимо ввести в командной строке терминала:
```txt
near contract \
    verify deployed-at \
    simple-package-verify-rs-ci.testnet \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO The code obtained from the contract account ID and the code calculated from the repository are the same.
 |    Contract code hash: 5KaX9FM9NtjpfahksL8TMWQk3LF7k8Sv88Qem4tGrVDW
 |    Contract version: 1.0.0
 |    Standards used by the contract:   [nep330:1.2.0]
 |    View the contract's source code on:       https://github.com/dj8yfo/verify_contracts_collection/tree/e3303f0cf8761b99f84f93c3a2d7046be6f4edb5
 |    Build Environment:        sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2
 |    Build Command:    cargo near build non-reproducible-wasm --locked
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/kRbwEWB7mnjBXULWPGcFE1597?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/kRbwEWB7mnjBXULWPGcFE1597.png" width="836"/>
</a>
</details>

#### download-abi - Download contract [ABI](https://github.com/near/abi)

Скачать ABI контракта возможно на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для получения ABI контракта необходимо ввести в командной строке терминала:

```txt
near contract \
    download-abi neardevhub-contract.volodymyr.testnet \
    save-to-file neardevhub-contract_volodymyr_testnet.abi.json \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO The file "neardevhub-contract_volodymyr_testnet.abi.json" was downloaded successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/fyOhP1ThQMX57JKcZBP72PJwl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/fyOhP1ThQMX57JKcZBP72PJwl.png" width="836"/>
</a>
</details>

#### download-wasm - Download wasm

Скачать файл контракта возможно на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для получения файла контракта необходимо ввести в командной строке терминала:

```txt
near contract \
    download-wasm volodymyr.testnet \
    save-to-file volodymyr_testnet.wasm \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO 
 |    The file "volodymyr_testnet.wasm" was downloaded successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/u9x4lbDFZu9rzwNgChu9jukGq?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/u9x4lbDFZu9rzwNgChu9jukGq.png" width="836"/>
</a>
</details>

#### view-storage - View contract storage state

Просмотреть значения ключей контракта возможно на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).
Сами же ключи можно просмотреть все (***all***) или отфильтрованные с помощью  ***keys-start-with-string*** или ***keys-start-with-bytes-as-base64***.

Для просмотра ключей контракта необходимо ввести в командной строке терминала:

```txt
near contract \
    view-storage turbo.volodymyr.testnet \
    all \
    as-json \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Contract state (values):
 |    [
 |      {
 |        "key": "MjF2b2xvZHlteXIudGVzdG5ldA==",
 |        "value": "JwAAAAAAAAAIAAAAAAAAAA=="
 |      },
 |      {
 |        "key": "U1RBVEU=",
 |        "value": ""
 |      },
 |      {
 |        "key": "ZnJvX3ZvbG9kLnRlc3RuZXQ=",
 |        "value": "HQAAAAAAAAAGAAAAAAAAAA=="
 |      },
 |      {
 |        "key": "dm9sb2R5bXlyLnRlc3RuZXQ=",
 |        "value": "dwEAAAAAAABLAAAAAAAAAA=="
 |      }
 |    ]

 INFO Contract state (proof):
 |    []
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/ng5lTKyfJaD0VGl9KCjWmDhjA?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ng5lTKyfJaD0VGl9KCjWmDhjA.png" width="836"/>
</a>
</details>

### transaction - Operate transactions

- [view-status](#view-status---View-a-transaction-status)
- [reconstruct-transaction](#reconstruct-transaction---Use-any-existing-transaction-from-the-chain-to-construct-NEAR-CLI-command-helpful-tool-for-re-submitting-similar-transactions)
- [construct-transaction](#construct-transaction---Construct-a-new-transaction)
- [sign-transaction](#sign-transaction---Sign-previously-prepared-unsigned-transaction)
- [send-signed-transaction](#send-signed-transaction---Send-a-signed-transaction)
- [print-transaction](#print-transaction---Print-all-fields-of-previously-prepared-transaction-without-modification)
- [send-meta-transaction](#send-meta-transaction---Act-as-a-relayer-to-send-a-signed-delegate-action-meta-transaction)

#### view-status - View a transaction status

Для просмотра статуса желаемой транзакции необходимо ввести в командной строке терминала её хэш:
```txt
near transaction \
    view-status GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Transaction status:
 |    RpcTransactionResponse {
 |        final_execution_outcome: Some(
 |            FinalExecutionOutcome(
 |                FinalExecutionOutcome {
 |                    status: SuccessValue(''),
 |                    transaction: SignedTransactionView {
 |                        signer_id: AccountId(
 |                            "volodymyr.testnet",
 |                        ),
 |                        public_key: ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS,
 |                        nonce: 165,
 |                        receiver_id: AccountId(
 |                            "qweqweqwe.volodymyr.testnet",
 |                        ),
 |                        actions: [
 |                            CreateAccount,
 |                            Transfer {
 |                                deposit: 100000000000000000000000000,
 |                            },
 |                            AddKey {
 |                                public_key: ed25519:AgVv8qjZ7yix3pTo7BimT1zoDYUSTGcg73RBssC5JMRf,
 |                                access_key: AccessKeyView {
 |                                    nonce: 0,
 |                                    permission: FullAccess,
 |                                },
 |                            },
 |                        ],
 |                        priority_fee: 0,
 |                        signature: ed25519:266jBRjvnaxe4mDyHRGwv3TJesvgRo2umJBqkZU26fRwmhVHciu3tBSLqRZFjEuqLTiwDTrFvfxpJ8Sbd2PqHHhv,
 |                        hash: GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank,
 |                    },
 |                    transaction_outcome: ExecutionOutcomeWithIdView {
 |                        proof: [],
 |                        block_hash: AQH6jDqqxpBYj5NSZv3Skg5hUZQRsn16jvDuphCTugSQ,
 |                        id: GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank,
 |                        outcome: ExecutionOutcomeView {
 |                            logs: [],
 |                            receipt_ids: [
 |                                5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh,
 |                            ],
 |                            gas_burnt: 424555062500,
 |                            tokens_burnt: 42455506250000000000,
 |                            executor_id: AccountId(
 |                                "volodymyr.testnet",
 |                            ),
 |                            status: SuccessReceiptId(5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh),
 |                            metadata: ExecutionMetadataView {
 |                                version: 1,
 |                                gas_profile: None,
 |                            },
 |                        },
 |                    },
 |                    receipts_outcome: [
 |                        ExecutionOutcomeWithIdView {
 |                            proof: [],
 |                            block_hash: DBUpiLVVDBQwSAPU8ZTE8KQnX5skDD1dTsBjJQ8kV24R,
 |                            id: 5DmuFwQaiSbEDiR7dx6sDurjyDyF92c1tK7gfN7bXqPh,
 |                            outcome: ExecutionOutcomeView {
 |                                logs: [],
 |                                receipt_ids: [
 |                                    851GMnZZ5FJ2aDSHM34N99yVb1ZkwY8n7F8rUcvuRpUU,
 |                                ],
 |                                gas_burnt: 424555062500,
 |                                tokens_burnt: 42455506250000000000,
 |                                executor_id: AccountId(
 |                                    "qweqweqwe.volodymyr.testnet",
 |                                ),
 |                                status: SuccessValue(''),
 |                                metadata: ExecutionMetadataView {
 |                                    version: 1,
 |                                    gas_profile: None,
 |                                },
 |                            },
 |                        },
 |                        ExecutionOutcomeWithIdView {
 |                            proof: [],
 |                            block_hash: BSjrH3WyKnXhD17drR94YfM725Ho59us9N4msXrrgHEw,
 |                            id: 851GMnZZ5FJ2aDSHM34N99yVb1ZkwY8n7F8rUcvuRpUU,
 |                            outcome: ExecutionOutcomeView {
 |                                logs: [],
 |                                receipt_ids: [],
 |                                gas_burnt: 0,
 |                                tokens_burnt: 0,
 |                                executor_id: AccountId(
 |                                    "volodymyr.testnet",
 |                                ),
 |                                status: SuccessValue(''),
 |                                metadata: ExecutionMetadataView {
 |                                    version: 1,
 |                                    gas_profile: None,
 |                                },
 |                            },
 |                        },
 |                    ],
 |                },
 |            ),
 |        ),
 |        final_execution_status: Final,
 |    }
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/OKLJdE09ueWy3HG27YcXExeF2?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/OKLJdE09ueWy3HG27YcXExeF2.png" width="836"/>
</a>
</details>

#### reconstruct-transaction - Use any existing transaction from the chain to construct NEAR CLI command (helpful tool for re-submitting similar transactions)

Рассмотрим пример, когда необходимо повторить выполненную ранее транзакцию:
```txt
near transaction \
    reconstruct-transaction GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Transaction GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank:
 |    signer_id:    volodymyr.testnet
 |    receiver_id:  qweqweqwe.volodymyr.testnet
 |    actions:
 |       -- create account:      qweqweqwe.volodymyr.testnet
 |       -- transfer deposit:    100 NEAR
 |       -- add access key:     
 |                       public key:   ed25519:AgVv8qjZ7yix3pTo7BimT1zoDYUSTGcg73RBssC5JMRf
 |                       nonce:        0
 |                       permission:   FullAccess

 INFO Here is your console command to run archive transaction. You can to edit it or re-run:
 |    ./near transaction construct-transaction volodymyr.testnet qweqweqwe.volodymyr.testnet add-action create-account add-action transfer '100 NEAR' add-action add-key grant-full-access use-manually-provided-public-key ed25519:AgVv8qjZ7yix3pTo7BimT1zoDYUSTGcg73RBssC5JMRf skip network-config testnet
```
</details>

<details><summary>Демонстрация работы команды в интерактивном режиме</summary>
<a href="https://asciinema.org/a/MCJgvWJpbu5W6ky1nxxkc38uW?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/MCJgvWJpbu5W6ky1nxxkc38uW.png" width="836"/>
</a>
</details>

#### construct-transaction - Construct a new transaction

Рассмотрим пример, когда необходимо выполнить несколько действий в рамках одной транзакции:
1. Создать аккаунт
2. Добавить созданному аккаунту ключи доступа
3. Осуществить перевод токенов на созданный аккаунт

Для этого воспользуемся конструктором транзакции:

<details><summary>Construct a new transaction (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/xph7SMTc2ZKlMCc8gbX179tvL?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/xph7SMTc2ZKlMCc8gbX179tvL.png" width="836"/>
</a>
</details>

#### sign-transaction - Sign previously prepared unsigned transaction

Рассмотрим пример, применив возможность создания транзакции в режиме _offline_:
1. Создать транзакцию.
2. При выборе средств подписи транзакции указать пункт _sign-later_ и следовать дальнейшим инструкциям.
3. Выведенная на экран транзакция в формате base64 может быть использована здесь для её подписи и/или последующей отправки.

<details><summary>Демонстрация работы команды в интерактивном режиме</summary>
<a href="https://asciinema.org/a/7yO1OobKvE3EWezUexPEHYYVC?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/7yO1OobKvE3EWezUexPEHYYVC.png" width="836"/>
</a>
</details>

#### send-signed-transaction - Send a signed transaction

Рассмотрим предыдущий пример, применив возможности отправки подписанной транзакции транзакции:
1. Создать транзакцию.
2. Подписать транзакцию своими ключами доступа.
3. Вывести подписанную транзакцию на экран в формате base64.
4. Отправить транзакцию.

<details><summary>Демонстрация работы команды в интерактивном режиме</summary>
<a href="https://asciinema.org/a/ignaXjJrvvDpQV4YUEK96iozX?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/ignaXjJrvvDpQV4YUEK96iozX.png" width="836"/>
</a>
</details>

#### print-transaction - Print all fields of previously prepared transaction without modification

Рассмотрим пример, когда необходимо просмотреть подписанную или неподписанную транзакцию. Воспользуемся подписанной транзакцией из предыдущего примера.
```txt
near transaction \
    print-transaction signed EQAAAHZvbG9keW15ci50ZXN0bmV0AJBJl9Sd5oZ1oXfc100pYXreoKWW+E63mVg8l69akbRWdqBxSFkvAAARAAAAZnJvX3ZvbG9kLnRlc3RuZXT7QjAtOsW6cL6MPyPnKpHqG2oBP2bjVLfbrdVzxhEvXwEAAAADAAAAoe3MzhvC0wAAAAAAAADBUbhi8hyWfMM9Q7pmTb9+2EJ2oUh+JAp6uVPLH2hnv4W7jHkPDAtUo4hnEiVYhPE8ztqJMJvEo3bT6wxISJkN
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
 INFO Signed transaction (full):
 |    signature:    ed25519:4sB6cgQqHnnx558AHjRSQ7uFLJCfi7J44kcYyhjB2DqZknKKQJbcFY8KqrzuWnJqQZNK56gt4hVLAvkTwd3zB3iU
 |    unsigned transaction hash (Base58-encoded SHA-256 hash): AaQPiqVRUSUjqQVyvrRUw7Sr7WDwMVr8h8UrZY1WhwqM
 |    public_key:   ed25519:AiEo5xepXjY7ChihZJ6AsfoDAaUowhPgvQp997qnFuRP
 |    nonce:        52060514001014
 |    block_hash:   HuosRMcXT7Ghe5ihdpiBRnDUqSZNLWZf4P2M4nQzGEsp
 |    signer_id:    volodymyr.testnet
 |    receiver_id:  fro_volod.testnet
 |    actions:
 |       -- transfer deposit:    1 NEAR
```
</details>

<details><summary>Демонстрация работы команды в интерактивном режиме</summary>
<a href="https://asciinema.org/a/D0JHl1RbLrHrAgjRnp61FAlBl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/D0JHl1RbLrHrAgjRnp61FAlBl.png" width="836"/>
</a>
</details>

#### send-meta-transaction - Act as a relayer to send a signed delegate action (meta-transaction)

Рассмотрим пример, применив возможности мета-транзакции:
1. Создать транзакцию.
2. Указать _network_ с поддержкой мета-транзакции.
3. Подписать транзакцию своими ключами доступа.
4. Вывести транзакцию на экран в формате base64 и передать её ретранслятору для отправки.

Отправить делегированную транзакцию:

<details><summary>Демонстрация работы команды в интерактивном режиме</summary>
<a href="https://asciinema.org/a/79Pwj2KxIHJgxC0CFrRTgfNcs?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/79Pwj2KxIHJgxC0CFrRTgfNcs.png" width="836"/>
</a>
</details>

### config - Manage connections in a configuration file

- [show-connections](#show-connections---Show-a-list-of-network-connections)
- [add-connection](#add-connection---Add-a-network-connection)
- [delete-connection](#delete-connection---Delete-a-network-connection)

#### show-connections - Show a list of network connections

Для просмотра данных конфигурационного файла (_config.toml_) можно воспользоваться интерактивным режимом либо ввести в командной строке терминала:
```txt
near config show-connections
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
credentials_home_dir = "/Users/frovolod/.near-credentials"
[network_connection.mainnet]
network_name = "mainnet"
rpc_url = "https://archival-rpc.mainnet.near.org/"
wallet_url = "https://wallet.near.org/"
explorer_transaction_url = "https://explorer.near.org/transactions/"
linkdrop_account_id = "near"

[network_connection.testnet]
network_name = "testnet"
rpc_url = "https://archival-rpc.testnet.near.org/"
wallet_url = "https://wallet.testnet.near.org/"
explorer_transaction_url = "https://explorer.testnet.near.org/transactions/"
linkdrop_account_id = "testnet"
faucet_url = "https://helper.nearprotocol.com/account"

[network_connection.pagoda-testnet]
network_name = "testnet"
rpc_url = "https://near-testnet.api.pagoda.co/rpc/v1/"
rpc_api_key = "c0a25b3c-39c2-4f62-a621-50e208b88e64"
wallet_url = "https://wallet.testnet.near.org/"
explorer_transaction_url = "https://explorer.testnet.near.org/transactions/"
linkdrop_account_id = "testnet"
faucet_url = "https://helper.nearprotocol.com/account"
meta_transaction_relayer_url = "https://near-testnet.api.pagoda.co/relay"
```
</details>

#### add-connection - Add a network connection

Для добавления данных о сети в конфигурационный файл (_config.toml_) можно воспользоваться интерактивным режимом либо ввести в командной строке терминала:
```txt
near config \
    add-connection \
        --network-name testnet \
        --connection-name pagoda-testnet \
        --rpc-url https://near-testnet.api.pagoda.co/rpc/v1/ \
        --wallet-url https://wallet.testnet.near.org/ \
        --explorer-transaction-url https://explorer.testnet.near.org/transactions/ \
        --rpc-api-key 'c0a25b3c-39c2-4f62-a621-50e208b88e64' \
        --linkdrop-account-id testnet \
        --faucet-url https://helper.nearprotocol.com/account \
        --meta-transaction-relayer-url https://near-testnet.api.pagoda.co/relay
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Configuration data is stored in a file "/Users/frovolod/Library/Application Support/near-cli/config.toml"
Network connection "pagoda-testnet" was successfully added to config.toml
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/49s6yuDmxQyaA2XgEqlBC6cpN?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/49s6yuDmxQyaA2XgEqlBC6cpN.png" width="836"/>
</a>
</details>

#### delete-connection - Delete a network connection

Для удаления сети из конфигурационного файла (_config.toml_) можно воспользоваться интерактивным режимом либо ввести в командной строке терминала:
```txt
near config delete-connection pagoda-testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Configuration data is stored in a file "/Users/frovolod/Library/Application Support/near-cli/config.toml"
Network connection "pagoda-testnet" was successfully removed from config.toml
```
</details>
