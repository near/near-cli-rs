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
        <a href="https://asciinema.org/a/30jHxm9lRevRG4K1h0GWlEciV?autoplay=1&t=1&speed=2">
            <img src="https://asciinema.org/a/30jHxm9lRevRG4K1h0GWlEciV.png" width="836"/>
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
- [add-key](#add-key---Add-an-access-key-to-an-account)
- [delete-key](#delete-key---Delete-an-access-key-from-an-account)
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/WA7eNU7hbmv7oa5lNLrmJzmRu?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/WA7eNU7hbmv7oa5lNLrmJzmRu.png" width="836"/>
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/oKL2H2gbDntOt0MHqpjsPnZZv?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/oKL2H2gbDntOt0MHqpjsPnZZv.png" width="836"/>
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/TqhSdwjoc9PMxbLZtTWSnCRR5?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/TqhSdwjoc9PMxbLZtTWSnCRR5.png" width="836"/>
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
The data for the access key is saved in macOS Keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/qEqxCxVMKjAWg92XhYCzWYhxO?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/qEqxCxVMKjAWg92XhYCzWYhxO.png" width="836"/>
</a>
</details>

#### using-seed-phrase - Import existing account using a seed phrase

Для авторизации пользователя необходимо ввести в командной строке терминала:
```txt
near account \
    import-account \
    using-seed-phrase 'rapid cover napkin accuse junk drill sick tooth poem patch evil fan' \
        --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
The data for the access key is saved in macOS Keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/G9l4So0zbT3bNGekePp1tzJg5?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/G9l4So0zbT3bNGekePp1tzJg5.png" width="836"/>
</a>
</details>

#### using-private-key - Import existing account using a private key

Для авторизации пользователя необходимо ввести в командной строке терминала:
```txt
near account \
    import-account \
    using-private-key ed25519:5YhAaEe3G4VtiBavJMvpzPPmknfsTauzVjwK1ZjPVw2MFM6zFyUv4tSiSfCbCn78mEnMifE6iX5qbhFsWEwErcC2 \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
The data for the access key is saved in macOS Keychain
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/KK14atSSbI8dLB3RcuyI2tfP8?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KK14atSSbI8dLB3RcuyI2tfP8.png" width="836"/>
</a>
</details>

#### export-account - Export existing account

- [using-web-wallet](#using-web-wallet---Export-existing-account-using-NEAR-Wallet-aka-sign-in)
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
<a href="https://asciinema.org/a/QqVhhVaBP4MP7XFDeb6arIB3S?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/QqVhhVaBP4MP7XFDeb6arIB3S.png" width="836"/>
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
The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/test_fro.testnet/ed25519_CCwvhsp3Y3BfLbfYJQJqXJA2CaSP7CRjn1t7PyEtsjej.json
The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/test_fro.testnet.json

New account <test_fro.testnet> created successfully.
Transaction ID: FnsrXbnzH1jjTWpAo1M8cZhEN5p7jyqgRPa1aqnRzxp3
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/FnsrXbnzH1jjTWpAo1M8cZhEN5p7jyqgRPa1aqnRzxp3
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/gKThQJT5rwgxLiN4EPQ1HiNnG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/gKThQJT5rwgxLiN4EPQ1HiNnG.png" width="836"/>
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
New account <test_fro1.testnet> created successfully.
Transaction ID: D1rRpZx5AcYWzC91Jdt69qF1iqai7knUAtvdvqNA2bv
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/D1rRpZx5AcYWzC91Jdt69qF1iqai7knUAtvdvqNA2bv
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/mYTEDj9Pxe3e6hwoTnDASuv0d?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/mYTEDj9Pxe3e6hwoTnDASuv0d.png" width="836"/>
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
New account <test_fro2.testnet> created successfully.
Transaction ID: E7rKjJiYg1BwXa6e7xMueDS8NUNjqZSN5zDRpB5sARTi
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/E7rKjJiYg1BwXa6e7xMueDS8NUNjqZSN5zDRpB5sARTi
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/uxZ7FVsK7OQTakfrgwHhL4X7D?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/uxZ7FVsK7OQTakfrgwHhL4X7D.png" width="836"/>
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
New account <test_fro3.testnet> created successfully.
Transaction ID: BStBXVisyR5FUj3ZfCAeQ1ohfwTnx2vTbYaRPLTQ5Uek
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/BStBXVisyR5FUj3ZfCAeQ1ohfwTnx2vTbYaRPLTQ5Uek
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
- [autogenerate-new-keypair](#autogenerate-new-keypair---Automatically-generate-a-key-pair)
- [use-manually-provided-seed-prase](#use-manually-provided-seed-prase---Use-the-provided-seed-phrase-manually)
- [use-manually-provided-public-key](#use-manually-provided-public-key---Use-the-provided-public-key-manually)
- [use-ledger](#use-ledger---Use-a-ledger)

##### autogenerate-new-keypair - Automatically generate a key pair

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
Transaction sent ...
New account <new.fro_volod.testnet> created successfully.
Transaction ID: DRT3EpCK9iT5APyGgfcgSoLPCLCYYKtnrVgDhGLDEZFo
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/DRT3EpCK9iT5APyGgfcgSoLPCLCYYKtnrVgDhGLDEZFo

The data for the access key is saved in a file /Users/frovolod/.near-credentials/testnet/new.fro_volod.testnet/ed25519_3ngtirechhepHKrzfkdgqqtwqSMtdbSLR6N1c4ivnzu6.json
The data for the access key is saved in a file "/Users/frovolod/.near-credentials/testnet/new.fro_volod.testnet.json"
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/62q0BKhCtXV8hQ3sxDpnO1CQl?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/62q0BKhCtXV8hQ3sxDpnO1CQl.png" width="836"/>
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
Transaction sent ...
New account <new7.testnet> created successfully.
Transaction ID: GxZRjmYxZyo6X6Mn1kfuRJhfUnxsUVCiHZAZKqrLtR27
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/GxZRjmYxZyo6X6Mn1kfuRJhfUnxsUVCiHZAZKqrLtR27

The data for the access key is saved in a file "/Users/frovolod/.near-credentials/testnet/new7.testnet/ed25519_EX1qK1S1T4WxXJFLH7qZvKxnGQtcKfEEsiA4BNxAZ6mP.json"
The file: /Users/frovolod/.near-credentials/testnet/new7.testnet.json already exists! Therefore it was not overwritten.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/MxmfDRdKPeP0VdXUiENmV2UXr?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/MxmfDRdKPeP0VdXUiENmV2UXr.png" width="836"/>
</a>
</details>

##### use-manually-provided-seed-prase - Use the provided seed phrase manually

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
<a href="https://asciinema.org/a/gEr7nG46C5kRp1DokYAQA28Qp?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/gEr7nG46C5kRp1DokYAQA28Qp.png" width="836"/>
</a>
</details>

##### use-manually-provided-public-key - Use the provided public key manually

Данная команда добавляет аккаунту заранее известный публичный ключ доступа.
Для выполнения этой команды необходимо ввести в командной строке терминала:
```txt
near account \
    create-account fund-myself pk.volodymyr.testnet '0.1 NEAR' \
    use-manually-provided-public-key ed25519:6jm8hWUgwoEeGmpdEyk9zrCqtXM8kHhvg8M236ZaGusS \
    sign-as volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
New account <pk.volodymyr.testnet> created successfully.
Transaction ID: CAVAR7jx2ofnbjxFFL2JVNbLsGNWF2q2tqMEtHxXmRLi
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/CAVAR7jx2ofnbjxFFL2JVNbLsGNWF2q2tqMEtHxXmRLi
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/R90IRnacRBO3Ni4PcpbRwm6Tt?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/R90IRnacRBO3Ni4PcpbRwm6Tt.png" width="836"/>
</a>
</details>

##### use-ledger - Use a ledger

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
Transaction sent ...
New account <ledger1.volodymyr.testnet> created successfully.
Transaction ID: BKJp3QdaLtnXA8xwfqyk6JfrDsDxbxqADVyuNzQmKGNL
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/BKJp3QdaLtnXA8xwfqyk6JfrDsDxbxqADVyuNzQmKGNL
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/SN2DNObpJeqI2QrN7BNjLNdU6?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/SN2DNObpJeqI2QrN7BNjLNdU6.png" width="836"/>
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
The file "/Users/frovolod/.near-credentials/implicit/1573066d3fa7a2d56357aa5ddbc84295d94c61590390000981f5900b04e2f55f.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/qPqMPP3tKwliWw2cu5vwCRfJi?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/qPqMPP3tKwliWw2cu5vwCRfJi.png" width="836"/>
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
The file "/Users/frovolod/.near-credentials/implicit/ledger/739c872c3057cd5d812c49345248b9fdd318c8ad33ace6cf0468109eae972c8e.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/kL5x9MXNrlSZWS83YjVkxnsf7?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/kL5x9MXNrlSZWS83YjVkxnsf7.png" width="836"/>
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
The file "/Users/frovolod/.near-credentials/implicit/eca9e1a6e0fa9a6af6d046bcffa6508f90f98e646836647ecd883d1d2b1989e5.json" was saved successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/rtmvhKL9eQXqIKBkvX62oi0qx?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/rtmvhKL9eQXqIKBkvX62oi0qx.png" width="836"/>
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
Profile for fro_volod.testnet updated successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/bF7AQuj012xVk4Xt5kMfOWAq1?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bF7AQuj012xVk4Xt5kMfOWAq1.png" width="836"/>
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
Profile for fro_volod.testnet updated successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/lbyMQp94TqvbNjBGmjQ49PEpJ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/lbyMQp94TqvbNjBGmjQ49PEpJ.png" width="836"/>
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
    delete-account 2.fro_volod.testnet \
    beneficiary volodymyr.testnet \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
Account <2.fro_volod.testnet> has been successfully deleted.
Transaction ID: EHvB47npN8Z46qhsrw5XpKmD3n3jDn4MGiD85YSqw7cy
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/EHvB47npN8Z46qhsrw5XpKmD3n3jDn4MGiD85YSqw7cy
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/bicRQEA5bhRG6e7nKaF8ghzVm?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bicRQEA5bhRG6e7nKaF8ghzVm.png" width="836"/>
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/KVfcCCyj2dEHEm4TcDkjtiW6s?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/KVfcCCyj2dEHEm4TcDkjtiW6s.png" width="836"/>
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
Transaction sent ...
Successful transaction
Added access key = ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 to fro_volod.testnet.
Transaction ID: 2oVDKopcWphN3qrUoq7XjFMpRuCUjz6jSU327q8trAQ5
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/2oVDKopcWphN3qrUoq7XjFMpRuCUjz6jSU327q8trAQ5
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/H4BfrteW1ClAzrLcRx9m8gQAV?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/H4BfrteW1ClAzrLcRx9m8gQAV.png" width="836"/>
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
Added access key = ed25519:27R66L6yevyHbsk4fESZDC8QUQBwCdx6vvkk1uQmG7NY to fro_volod.testnet.
Transaction ID: DaJySrNtSUZU7KPyvfUMbh6xYi9vZeMvnj4Umo7ZzdB3
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/DaJySrNtSUZU7KPyvfUMbh6xYi9vZeMvnj4Umo7ZzdB3
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/h08oydOTq3njf6mt1FNRMHGVs?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/h08oydOTq3njf6mt1FNRMHGVs.png" width="836"/>
</a>
</details>

#### delete-key - Delete an access key from an account

Для удаления ключей доступа необходимо ввести в командной строке терминала:
```txt
near account \
    delete-key fro_volod.testnet \
    ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3 \
    network-config testnet \
    sign-with-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
Access key <ed25519:75a5ZgVZ9DFTxs4THtFxPtLj7AY3YzpxtapTQBdcMXx3> for account <fro_volod.testnet> has been successfully deleted.
Transaction ID: 6S7bJ76QNFypUvP7PCB1hkLM7X5GxPxP2gn4rnDHMzPz
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/6S7bJ76QNFypUvP7PCB1hkLM7X5GxPxP2gn4rnDHMzPz
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/IYaNEYcMHtmSe6zKc2L63Okph?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/IYaNEYcMHtmSe6zKc2L63Okph.png" width="836"/>
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
    sign-with-macos-keychain \
    send
```

<details><summary><i>Результат выполнения команды</i></summary>

```txt
<fro_volod.testnet> has successfully added a deposit of 1 NEAR to <volodymyr.testnet> on contract <v1.social08.testnet>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/QXXvnhz2HasKtQdT5KPVr6d1n?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/QXXvnhz2HasKtQdT5KPVr6d1n.png" width="836"/>
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
<volodymyr.testnet> has successfully withdraw 0.5 NEAR from <v1.social08.testnet>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
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
Transaction sent ...
Successful transaction
<fro_volod.testnet> has transferred 0.1 NEAR to <volodymyr.testnet> successfully.
Transaction ID: 8BbB674VDxeg36egMzdHFsCUExpkLWAWeYqEfd9u9ZaD
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/8BbB674VDxeg36egMzdHFsCUExpkLWAWeYqEfd9u9ZaD
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/U1pNSHZw812e4BHvnFGpefVs4?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/U1pNSHZw812e4BHvnFGpefVs4.png" width="836"/>
</a>
</details>

#### send-ft - The transfer is carried out in FT tokens

Данная команда служит для перевода средств в FT токенах между аккаунтами. Обратите внимание, что количество пересылаемых токенов указывается в безразмерных единицах.
Для выполнения этой команды необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
The "ft_transfer" call to <usdn.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 5a7YmANdpimiqUm6WC6n4dd91b6A9PafNNhad8HWKugN
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/5a7YmANdpimiqUm6WC6n4dd91b6A9PafNNhad8HWKugN
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/uvZGcJUpufJZdB10GsQlfXwW1?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/uvZGcJUpufJZdB10GsQlfXwW1.png" width="836"/>
</a>
</details>

#### send-nft - The transfer is carried out in NFT tokens

Данная команда служит для перевода средств в NFT токенах между аккаунтами.
Для выполнения этой команды необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
The "nft_transfer" call to <paras-token-v2.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 9q2VbakZbj5ja6GAFXpFnbtbYHijEHyT7Ry34GQ6cvLB
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/9q2VbakZbj5ja6GAFXpFnbtbYHijEHyT7Ry34GQ6cvLB
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/iFXW6ryGQSdsWML0c3qAw3qGY?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/iFXW6ryGQSdsWML0c3qAw3qGY.png" width="836"/>
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
fro_volod.testnet account has 169.589001320890476999999994 NEAR available for transfer (the total balance is 172.482461320890476999999994 NEAR, but 2.89246 NEAR is locked for storage and the transfer transaction fee is ~0.001 NEAR)
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/lKGalzAxt3zCSxOsreqdykO8X?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/lKGalzAxt3zCSxOsreqdykO8X.png" width="836"/>
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
fro_volod.testnet account has "31942967677775774595" FT tokens (FT-contract: usdn.testnet)
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/k7Bz5r20x2Bo5RIX7Q1VnpNZC?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/k7Bz5r20x2Bo5RIX7Q1VnpNZC.png" width="836"/>
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/znmY5yzIlSTjOlRRRUHzeeuzJ?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/znmY5yzIlSTjOlRRRUHzeeuzJ.png" width="836"/>
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

- [view-balance](#View-the-total-balance-for-a-given-account)
- [deposit-and-stake](#deposit-and-stake---Deposits-the-attached-amount-into-the-inner-account-of-the-predecessor-and-stakes-it)
- [stake](#stake---Staking-the-given-amount-from-the-inner-account-of-the-predecessor)
- [stake-all](#stake-all---Staking-all-available-unstaked-balance-from-the-inner-account-of-the-predecessor)
- [unstake](#unstake---Unstaking-the-given-amount-from-the-inner-account-of-the-predecessor)
- [unstake-all](#unstake-all---Unstaking-all-staked-balance-from-the-inner-account-of-the-predecessor)
- [withdraw](#withdraw---Withdrawing-the-non-staked-balance-for-given-account)
- [withdraw-all](#withdraw-all---Withdrawing-the-entire-unstaked-balance-from-the-predecessor-account)

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
Delegated stake balance with validator <aurora.pool.f863973.m0> by <volodymyr.testnet>:
      Staked balance:           38.021465232511349340052266 NEAR
      Unstaked balance:          0.000000000000000000000001 NEAR
      Total balance:            38.021465232511349340052267 NEAR
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/2ZFe7ILJOoJCHYPYSnv7JBBFy?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/2ZFe7ILJOoJCHYPYSnv7JBBFy.png" width="836"/>
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
<volodymyr.testnet> has successfully deposited and staked 15 NEAR on <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/tTwzlj0FszzXEh36aP6ZaTdhG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/tTwzlj0FszzXEh36aP6ZaTdhG.png" width="836"/>
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
<volodymyr.testnet> has successfully stake 5 NEAR on <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/10T84aFMiJSYLv3shBEsql68L?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/10T84aFMiJSYLv3shBEsql68L.png" width="836"/>
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
<volodymyr.testnet> has successfully stake the entire amount on <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/bhUAfnDCnt9U2XQLeY46sbTWR?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/bhUAfnDCnt9U2XQLeY46sbTWR.png" width="836"/>
</a>
</details>

##### unstake - Unstake a certain amount of delegated NEAR tokens from a avalidator's staking pool

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
<volodymyr.testnet> has successfully unstake 7 NEAR from <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/vOJusmeGFwrofAKN6wd2Q3a5w?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/vOJusmeGFwrofAKN6wd2Q3a5w.png" width="836"/>
</a>
</details>

##### unstake-all - Unstake all delegated NEAR tokens from a avalidator's staking pool

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
<volodymyr.testnet> has successfully unstake the entire amount from <aurora.pool.f863973.m0>.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/FgEjwrSlSHjIXeUBZGyl1O6vG?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/FgEjwrSlSHjIXeUBZGyl1O6vG.png" width="836"/>
</a>
</details>

##### withdraw - Withdraw a certain amount of unstaked NEAR tokens from a avalidator's staking pool

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
Error:
   0: <volodymyr.testnet> can't withdraw tokens in the current epoch.
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/U4V6CUJU0vG0dJhT4igXyrJpk?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/U4V6CUJU0vG0dJhT4igXyrJpk.png" width="836"/>
</a>
</details>

##### withdraw-all - Withdraw all unstaked NEAR tokens from a avalidator's staking pool

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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/OHhdlJEaoA4nLJSDtybgc7kCR?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/OHhdlJEaoA4nLJSDtybgc7kCR.png" width="836"/>
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
Transaction sent ...
Successful transaction
The "rate" call to <turbo.volodymyr.testnet> on behalf of <fro_volod.testnet> succeeded.
Transaction ID: 7RuoSAdCctSEw63GKsfQJg1YXRzH3msUCo4oygzauPko
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/7RuoSAdCctSEw63GKsfQJg1YXRzH3msUCo4oygzauPko
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/S6LHwINBHskznxMrJPHzUmgxM?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/S6LHwINBHskznxMrJPHzUmgxM.png" width="836"/>
</a>
</details>

#### deploy - Add a new contract code

Для добавления нового контракта необходимо ввести в командной строке терминала:
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

<details><summary><i>Результат выполнения команды</i></summary>

```txt
Transaction sent ...
Successful transaction
Contract code has been successfully deployed.
The "increment" call to <262.volodymyr.testnet> on behalf of <262.volodymyr.testnet> succeeded.
Transaction ID: 4YGGhF88aevNGpF5uaXNGHfQprHRqkia7eTpyxegJVms
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/4YGGhF88aevNGpF5uaXNGHfQprHRqkia7eTpyxegJVms
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/7KD9gM9tj2AWtgGpjUmytkPg9?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/7KD9gM9tj2AWtgGpjUmytkPg9.png" width="836"/>
</a>
</details>

#### download-wasm - Download wasm

Скачать файл контракта возможно на текущий момент времени (***now***) и на определеный момент в прошлом, указав блок (***at-block-height*** или ***at-block-hash***).
Примеры использования этих параметров рассмотрены в разделе [View properties for an account](#view-account-summary---view-properties-for-an-account).

Для получения файла контракта необходимо ввести в командной строке терминала:

```txt
near contract \
    download-wasm 262.volodymyr.testnet \
    to-folder /Users/frovolod/Downloads \
    network-config testnet \
    now
```

<details><summary><i>Результат выполнения команды</i></summary>
```txt
The file "/Users/frovolod/Downloads/contract_262_volodymyr_testnet.wasm" was downloaded successfully
```
</details>

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/2UbeTzLJq16qtCUR015wuRFmN?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/2UbeTzLJq16qtCUR015wuRFmN.png" width="836"/>
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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
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

Для просмотра статуса желаемой транзакции необходимо ввести в командной строке терминала её хэш:
```txt
near transaction \
    view-status GDoinMecpvnqahzJz9tXLxYycznL4cAoxKTPEnJZ3ank \
    network-config testnet
```

<details><summary><i>Результат выполнения команды</i></summary>

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

<details><summary><i>Демонстрация работы команды в интерактивном режиме</i></summary>
<a href="https://asciinema.org/a/xf69gJEha7yO27E27CZszkN97?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/xf69gJEha7yO27E27CZszkN97.png" width="836"/>
</a>
</details>

#### construct-transaction - Construct a new transaction

Рассмотрим пример, когда необходимо выполнить несколько действий в рамках одной транзакции:
1. Создать аккаунт
2. Добавить созданному аккаунту ключи доступа
3. Осуществить перевод токенов на созданный аккаунт

Для этого воспользуемся конструктором транзакции:

<details><summary>Construct a new transaction (демонстрация работы команды)</summary>
<a href="https://asciinema.org/a/WNbxN1GB861q2sBbiKbQyVl3S?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/WNbxN1GB861q2sBbiKbQyVl3S.png" width="836"/>
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
