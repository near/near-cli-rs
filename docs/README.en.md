near-cli
--------
near-cli is a command line utility for working with the Near Protocol blockchain.

## README.md

* ru [Русский](README.ru.md)

## Documentation quick links

- [Usage](#usage)
- [Installation](#installation)
- [User Guide](#user-guide)
- [Config](#config)
- [Building](#building)

## Usage

In general, it is difficult for a beginner to immediately understand how commands work.  
For example, I consider having the following command to do a transfer:
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
This is the complete version of the command. The result of this command will be as follows:
```txt
Successful transaction
<volodymyr.testnet> has transferred 1 NEAR to <fro_volod.testnet> successfully.
Transaction ID: G4t6Sgz2FjnNpruYjPP1ZJAKfRmBffVaqmj8Nup2TaAg
To see the transaction in the transaction explorer, please open this url in your browser:
https://explorer.testnet.near.org/transactions/G4t6Sgz2FjnNpruYjPP1ZJAKfRmBffVaqmj8Nup2TaAg
```
Typing this or another command manually, you can make a mistake or forget the sequence of the command.  
It is not a problem. `--help` will tell you how to build a command properly.  
However, using _near-cli_, you can press _Enter_ anywhere in the command line and the interactive mode of the program will continue to compose the command from the place where you finished entering the necessary parameters.

<details><summary><i>Demonstration of the utility with a partially recruited command</i></summary>
<a href="https://asciinema.org/a/AfxLN1QtJi1z1qXuowTj2nDw2?autoplay=1&t=1&speed=2">
    <img src="https://asciinema.org/a/AfxLN1QtJi1z1qXuowTj2nDw2.png" width="836"/>
</a>
</details>

## Installation

At this stage of the development of the utility, installation of the program is not required.  
It is enough to [download](https://github.com/FroVolod/near-cli/releases/) the archive file suitable for your operating system installed on the computer and unzip it.  
The resulting directory contains the executable file _near-cli_, which is accompanied by detailed [user guide](GUIDE.en.md).

## User Guide

Detailed user guide is available in the file [GUIDE.en.md](GUIDE.en.md).

## Config

The directory with access keys and available connection networks are defined in the configuration file (`near-cli/config.toml`), which is located depending on the operating system in the following places:

- macOS: `$HOME/Library/Application Support` (e.g. `/Users/Alice/Library/Application Support`)
- Linux: `$XDG_CONFIG_HOME` or `$HOME/.config` (e.g. `/home/alice/.config`)
- Windows: `{FOLDERID*RoamingAppData}` (e.g. `C:\Users\Alice\AppData\Roaming`)

You can learn more about working with the configuration file [here](GUIDE.en.md#config---manage-connections-in-a-configuration-file).

## Building

_near-cli_ is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
_near-cli_ compiles with Rust 1.64.0 (stable) or newer. In general, _near-cli_ tracks the latest stable release of the Rust compiler.

To build _near-cli_:

```txt
$ git clone https://github.com/near/near-cli-rs.git
$ cd near-cli-rs
$ cargo build --release
$ ./target/release/near-cli --version
near-cli 0.2.0
```
