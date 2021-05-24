## Инструкция

Это руководство предназначено для того, чтобы дать подробное описание утилиты near-cli и
обзор её возможностей. Предполагается, что утилита near-cli
[установлена](README.ru.md#installation)
и пользователи знакомы с использованием инструментов командной строки. Также предполагается Unix-подобная система, хотя большинство команд, вероятно, легко
переводимы в любую среду оболочки командной строки.

### Группы команд

* [Add access key, contract code, stake proposal, sub-account, implicit-account](#add-access-key-contract-code-stake-proposal-sub-account-implicit-account)
* [Construct a new transaction](#construct-a-new-transaction)
* [Delete access key, account](#delete-access-key-account)
* [Execute function (contract method)](#execute-function-contract-method)
* [Transfer tokens](#transfer-tokens)
* [Helpers](#helpers)
* [View account, contract code, contract state, transaction](#view-account-contract-code-contract-state-transaction)


### Add access key, contract code, stake proposal, sub-account, implicit-account

__1. Add a new access key for an account__

Выполним команду с такими условиями:
  * публичный ключ доступа будет введен вручную
  * ключи будут иметь полный доступ

Для выполнения этой команды в командной строке терминала необходимо ввести:
```
./near-cli add access-key \
        network testnet \
        account 'volodymyr.testnet' \
        public-key 'ed25519:Ebx7...' \
        grant-full-access \
        sign-with-keychain \
        send
```
Результат выполнения команды:
```
========= SENT =========


---  Signed transaction:   ---
    SignedTransaction {
    transaction: Transaction {
        signer_id: "volodymyr.testnet",
        public_key: ed25519:7FmD...,
        nonce: 149,
        receiver_id: "volodymyr.testnet",
        block_hash: `Am5ZGCMSeEyY4BJqqwMBKA9AV8uB77m5Yn52P4rbEFu6`,
        actions: [
            AddKey(
                AddKeyAction {
                    public_key: ed25519:Ebx7...,
                    access_key: AccessKey {
                        nonce: 0,
                        permission: FullAccess,
                    },
                },
            ),
        ],
    },
    signature: ed25519:2iqJLi9K6kTtkTR1e4dVXJfa1wYN5Js34WtufurJDzfGy9SXvAnxiDXKAUYey1CFi3xTQDsHwKDYvELswWHfV8EY,
    hash: `Fjt8PQtmk6HiFz59sA1wnbDmUvKmUBTYmSkwT4wMSgct`,
    size: 162,
}


---  serialize_to_base64:   --- 
   "EQAAAHZvbG9keW15ci50ZXN0bmV0AFzuPvN68GwMEHmmSd/z+SfoSEHUz9773txWhikaAcDPlQAAAAAAAAARAAAAdm9sb2R5bXlyLnRlc3RuZXSRA+PDDBgYWU9gZ3tQIeY9mOpLdU/AofbhOJ+e3ZFGTQEAAAAFAMob/ZK9JLsyX0GsR1RyW9L2ZAclRYRiqIZwzCPP7dmEAAAAAAAAAAABAFYXixaHv0sQnm9oNnoSTV1tIKOa5nTf3BRr2Lxn4dHXLbVTB6WBjJHh10mRVoXxmqoE5JhiVpY3/U/oXgYoHg0="


---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
 }
```
<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/2hsXeOrB3Kt13DSTDC5BVcqau?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/2hsXeOrB3Kt13DSTDC5BVcqau.png" width="836"/>
</a>
</details>

Изменим наши парамерты для добавленного кюча:
  * ключи будут сгенерированы автоматически
  * будут иметь функциональный доступ
  * транзакция будет подписана вручную

Для этого введем следующую команду:
```
near-cli add access-key \
        network testnet \
        account 'volodymyr.testnet' \
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
Результат выполнения команды:
```
========= SENT =========


---  Signed transaction:   ---
    SignedTransaction {
    transaction: Transaction {
        signer_id: "21.volodymyr.testnet",
        public_key: ed25519:Ebx7NiwqupsshnUsEZCzgm84SQqi8LZWJx7ermrr14JF,
        nonce: 19,
        receiver_id: "21.volodymyr.testnet",
        block_hash: `54idHezkbgmzcmpBdCH5Fksr4gZHRamdV9UWeBUG3mf1`,
        actions: [
            AddKey(
                AddKeyAction {
                    public_key: ed25519:4YDJbW2GDDgzgNUW5UmC7iDxEy8e2JJenKbUyUMUxhzG,
                    access_key: AccessKey {
                        nonce: 0,
                        permission: FunctionCall(
                            FunctionCallPermission {
                                allowance: Some(
                                    10000000000000000000000000,
                                ),
                                receiver_id: "meta.pool.testnet",
                                method_names: [
                                    "set_a",
                                    " set_b",
                                ],
                            },
                        ),
                    },
                },
            ),
        ],
    },
    signature: ed25519:6EvuDd9GsZEqUgnr9KPaRv3TexVz4rPdEJ6MiorCFvfpd5bCVKHAxAdvYQdL7n76sr4NFDZrmtXhJmuHreAMdDv,
    hash: `CAKAx4MGnmvCbFiuRfjNZpvivdbtvMWw15ftDJAbzqAa`,
    size: 229,
}


---  serialize_to_base64:   --- 
   "FAAAADIxLnZvbG9keW15ci50ZXN0bmV0AMob/ZK9JLsyX0GsR1RyW9L2ZAclRYRiqIZwzCPP7dmEEwAAAAAAAAAUAAAAMjEudm9sb2R5bXlyLnRlc3RuZXQ8Yo5v35DY5uHEw5CTtRQycg1L8uIrXLMhASkPPkHI7AEAAAAFADSR+jEvIqz/Mmw2d7LyuyJIWd3pdV72ZFx+sX7CTw4hAAAAAAAAAAAAAQAAAEpIARQWlUUIAAAAAAARAAAAbWV0YS5wb29sLnRlc3RuZXQCAAAABQAAAHNldF9hBgAAACBzZXRfYgAEhNenJczDVbUDfaxLFyd5Vo5/PIROxP9IuyFE9aE1n9G4FeGT23KBBf8z/HMok6ebbmarbmm5BLlmBK9UlEcN"


---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
 }
```
<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/s9Z0eRw9fuxTrRDSTvpzcNZGo?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/s9Z0eRw9fuxTrRDSTvpzcNZGo.png" width="836"/>
</a>
</details>

__2. Add a new contract code__
<details><summary>Add a new contract code (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

__3. Add an implicit-account__
<details><summary>Add an implicit-account (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

__4. Add a new stake proposal__
<details><summary>Add a new stake proposal (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

__5. Add a new sub-account__
<details><summary>Add a new sub-account (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

<!-- * [Add a new access key for an account](#add-access-key) -->
<!-- * [Add a new contract code](#add-contract-code)
* [Add an implicit-account](#add-implicit-account)
* [Add a new stake proposal](#add-stake-proposal)
* [Add a new sub-account](#add-sub-account) -->


### Construct a new transaction

Рассмотрим пример, когда необходимо:
1. Создать аккаунт
2. Добавить созданному аккаунту ключи доступа
3. Осуществить перевод токенов на созданный аккаунт

Для этого воспользуемся конструктором транзакции:


<!-- <details><summary>Construct a new transaction</summary>
<p>
</p><pre><code>
</code></pre>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
<p></p>
</details> -->

<details><summary>Construct a new transaction (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

### Delete access key, account


### Execute function (contract method



### Transfer tokens



### Helpers



### View account, contract code, contract state, transaction


