near-cli
--------
_near-cli_ – это утилита командной строки для работы с блокчейном Near Protocol.

## README.md

- en [English](README.en.md)

## Оглавление

- [Применение](#применение)
- [Установка](#установка)
- [Инструкция](#инструкция)
- [Конфигурационный файл](#конфигурационный-файл)
- [Сборка](#сборка)

## Применение

Вцелом новичку трудно сразу разобраться как устроены команды.  
Например, для осуществления перевода токенов необходимо набрать в терминале такую команду:

```txt
./near-cli tokens \
    'volodymyr.testnet' \
    send-near \
    'fro_volod.testnet' \
    '1 NEAR' \
    network testnet \
    sign-with-keychain \
    send
```

Это полная команда. Результат ее работы будет такой:

```txt
Successful transaction
<volodymyr.testnet> has transferred 1 NEAR to <fro_volod.testnet> successfully.
Transaction ID: G4t6Sgz2FjnNpruYjPP1ZJAKfRmBffVaqmj8Nup2TaAg
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/G4t6Sgz2FjnNpruYjPP1ZJAKfRmBffVaqmj8Nup2TaAg
```

Набирая эту или другую команду вручную, Вы можете допустить ошибку, либо забыть последовательность набора команды.  
Не проблема, `--help` подскажет как правильно выстроить команду.  
Однако, используя _near-cli_, Вы __в любом месте набора команды__ можете нажать Enter и интерактивный режим программы продолжит работу по составлению команды с того места, где Вы закончили вводить необходимые параметры.

<details><summary><i>Демонстрация работы утилиты с частично набранной командой</i></summary>
<a href="https://asciinema.org/a/AfxLN1QtJi1z1qXuowTj2nDw2?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/AfxLN1QtJi1z1qXuowTj2nDw2.png" width="836"/>
</a>
</details>

## Установка

На данном этапе разработки утилиты установка программы не требуется.  
Достаточно [загрузить](https://github.com/FroVolod/near-cli/releases/) архивный файл, подходящий к Вашей операциой системе, установленной на компьютере, и разархивировать его.  
В полученном каталоге находится исполняемый файл _near-cli_, к которому прилагается [подробная инструкция для пользователя](GUIDE.ru.md).  
Если необходимо скомпилировать CLI из исходного кода, перейдите к разделу [Сборка](#сборка).

## Инструкция

Подробная инструкция доступна в файле [GUIDE.ru.md](GUIDE.ru.md).

## Конфигурационный файл

Каталог с ключами доступа и доступные сети подключения определены в конфигурационном файле (`near-cli/config.toml`), который находится в зависимости от операциооной системы в следующих местах:

- macOS: `$HOME/Library/Application Support` (например, `/Users/Alice/Library/Application Support`)
- Linux: `$XDG_CONFIG_HOME` или `$HOME/.config` (например, `/home/alice/.config`)
- Windows: `{FOLDERID*RoamingAppData}` (например, `C:\Users\Alice\AppData\Roaming`)

Подробнее о работе с конфигурационным файлом можно ознакомиться [здесь](GUIDE.ru.md#config---manage-connections-in-a-configuration-file).

## Сборка

_near-cli_ написан на Rust. Поэтому необходимо
[установить Rust](https://www.rust-lang.org/) для компиляции программы.
_near-cli_ компилируется на версии Rust 1.64.0 (stable) или новее.

Сборка _near-cli_:

```txt
$ git clone https://github.com/near/near-cli-rs.git
$ cd near-cli-rs
$ cargo build --release
$ ./target/release/near-cli --version
near-cli 0.2.0
```
