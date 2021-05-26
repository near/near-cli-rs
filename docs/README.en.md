near-cli
--------
near-cli is a command line utility for working with the Near Protocol blockchain.

## README.md

* ru [Русский](README.ru.md)

## Documentation quick links

* [Usage](#usage)
* [User Guide](#user-guid)
* [Installation](#installation)
* [Building](#building)

## Usage

In general, it is difficult for a beginner to immediately understand how commands work.  
For example, I consider having the following command to do a transfer:
```txt
near-cli transfer near \
    network testnet \
    sender 'volodymyr.testnet' \
    receiver '21.volodymyr.testnet' \
    amount  '1 NEAR' \
    sign-with-keychain \
    send
```
This is the complete version of the command. The result of this command will be as follows:
```txt
---  Success:  ---
 FinalExecutionOutcome {
    status: SuccessValue(``),
    ...
}
```
Typing this or another command manually, you can make a mistake or forget the sequence of the command.  
It is not a problem. `--help` will tell you how to build a command properly.  
However, using near-cli, you can press _Enter_ anywhere in the command line and the interactive mode of the program will continue to compose the command from the place where you finished entering the necessary parameters.

<details><summary><i>Demonstration of the utility with a partially recruited command</i></summary>
<a href="https://asciinema.org/a/tdNu6qoDKUzFH6ZCsfADHoqOP?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/tdNu6qoDKUzFH6ZCsfADHoqOP.png" width="836"/>
</a>
</details>


## User Guide

Detailed user guide is available in the file [GUIDE.en.md](GUIDE.en.md).

## Installation

At this stage of the development of the utility, installation of the program is not required.  
It is enough to [download](https://github.com/FroVolod/near-cli/releases/) the archive file suitable for your operating system installed on the computer and unzip it.  
The resulting directory contains the executable file _near-cli_, which is accompanied by detailed [user guide](GUIDE.en.md).

### Building

near-cli is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
near-cli compiles with Rust 1.50.0 (stable) or newer. In general, near-cli tracks the latest stable release of the Rust compiler.

To build near-cli:

```txt
$ git clone https://github.com/FroVolod/near-cli
$ cd near-cli
$ cargo build --release
$ ./target/release/near-cli --version
near-cli 0.1.0
```
