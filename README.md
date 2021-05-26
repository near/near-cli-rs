near-cli
--------
near-cli – это утилита командной строки для работы с блокчейном Near Protocol. 

### README.md

* en [English](docs/README.en.md)
* ru [Русский](docs/README.ru.md)


### Usage

Вцелом новичку трудно сразу разобраться как устроены команды.
For example, I consider having the following command to do a transfer:
```sh
near-cli transfer near \
    network testnet \
    sender 'volodymyr.testnet' \
    receiver '21.volodymyr.testnet' \
    amount  '1 NEAR' \
    sign-with-keychain \
    send
```
Это полная команда. Результат ее работы будет такой:
```
---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
}
```
Набирая эту или другую команду вручную, Вы можете допустить ошибку, либо забыть последовательность набора команды.
Не проблема. – help подскажет как правильно выстроить команду.
Однако, используя near-cli, Вы в любом месте набора команды можете нажать Enter и интерактивный режим программы продолжит работу по составлению команды с того места, где Вы закончили вводить необходимые параметры.





### Installation

### Building

near-cli is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
near-cli compiles with Rust 1.50.0 (stable) or newer. In general, near-cli tracks the latest stable release of the Rust compiler.

To build near-cli:

```
$ git clone https://github.com/FroVolod/near-cli
$ cd near-cli
$ cargo build --release
$ ./target/release/near-cli --version
near-cli
```
