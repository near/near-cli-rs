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

* __Add a new access key for an account__

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
<details><summary>Add a new access key for an account (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

* __Add a new contract code__
<details><summary>Add a new contract code (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

* __Add an implicit-account__
<details><summary>Add an implicit-account (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

* __Add a new stake proposal__
<details><summary>Add a new stake proposal (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/9kuNItY3K5ee116ReSvrOnb4R.png" width="836"/>
</a>
</details>

* __Add a new sub-account__
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


