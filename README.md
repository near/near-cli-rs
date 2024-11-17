# NEAR CLI

near CLI is your **human-friendly** companion that helps to interact with [NEAR Protocol](https://near.org) from command line.

Just run `near` and let it guide you through!

<p>
  <img src="docs/media/create-account.svg" alt="" width="1200">
</p>

## Install

Visit [Releases page](https://github.com/near/near-cli-rs/releases/) to see the latest updates.

<details>
  <summary>Install via Windows Installer (Windows)</summary>

  
https://github.com/user-attachments/assets/607f797b-0412-4741-984b-6b6032d05262

</details>

<details>
  <summary>Install via powershell script (Windows)</summary>

```sh
irm https://github.com/near/near-cli-rs/releases/latest/download/near-cli-rs-installer.ps1 | iex
```

https://github.com/user-attachments/assets/7d5d090e-4885-4c27-9d0f-045905952071

</details>

<details>
  <summary>Install via shell script (macOS, Linux, Windows/WSL)</summary>

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/latest/download/near-cli-rs-installer.sh | sh
```
</details>

<details>
  <summary>Run via npx (Node.js on Windows, Linux, macOS)</summary>

```sh
npx near-cli-rs
```
</details>

<details>
  <summary>Use in package.json scripts (Node.js on Windows, Linux, macOS)</summary>

```sh
npm install near-cli-rs
```
</details>

<details>
  <summary>Get a portable version (Windows, Linux, macOS)</summary>

  
https://github.com/user-attachments/assets/4a7e4633-1957-4dc2-a032-827fa9c06c29

</details>

<details>
  <summary>Compile and install from the source code (Cargo on Windows, Linux, macOS)</summary>

Install it with `cargo`, just make sure you have [Rust](https://rustup.rs) installed on your computer.

```bash
cargo install near-cli-rs
```

or, install the most recent version from git repository:

```bash
$ cargo install --git https://github.com/near/near-cli-rs
```
</details>

<details>
  <summary>Install on CI (GitHub Actions)</summary>

It is often desirable to use `near` CLI from CI to automate some actions, so here is an example of how you can make a function call during CI:

```yml
name: Release
on:
  push:
    branches: [main]

jobs:
  deploy-widgets:
    runs-on: ubuntu-latest
    name: Make a function call on mainnet
    env:
      NEAR_NETWORK_CONNECTION: mainnet
      NEAR_CONTRACT_ACCOUNT_ID: ${{ vars.NEAR_CONTRACT_ACCOUNT_ID }}
      NEAR_SIGNER_ACCOUNT_ID: ${{ vars.NEAR_SIGNER_ACCOUNT_ID }}
      NEAR_SIGNER_ACCOUNT_PUBLIC_KEY: ${{ vars.NEAR_SIGNER_ACCOUNT_PUBLIC_KEY }}
      NEAR_SIGNER_ACCOUNT_PRIVATE_KEY: ${{ secrets.NEAR_SIGNER_ACCOUNT_PRIVATE_KEY }}

    steps:
    - name: Checkout repository
      uses: actions/checkout@v2

    - name: Install near CLI
      run: |
        curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/download/v0.7.4/near-cli-rs-installer.sh | sh

    - name: Call some function
      run: |
        near contract call-function as-transaction "$NEAR_CONTRACT_ACCOUNT_ID" 'function_name_here' json-args '{}' prepaid-gas '100 TeraGas' attached-deposit '0 NEAR' sign-as "$NEAR_SIGNER_ACCOUNT_ID" network-config "$NEAR_NETWORK_CONNECTION" sign-with-plaintext-private-key --signer-public-key "$NEAR_SIGNER_ACCOUNT_PUBLIC_KEY" --signer-private-key "$NEAR_SIGNER_ACCOUNT_PRIVATE_KEY" send
```

You will need to configure GitHub Actions Secrets and Variables and once it is ready, this CI will only take a couple of _seconds_ to complete!

See how it is used by [DevHub]([https://github.com/near/devgigsboard](https://github.com/NEAR-DevHub/neardevhub-contract/blob/05fb66ac307d84347f29e8e3ab9f429a78cb6513/.github/workflows/release.yml#L30-L41)).
</details>

## Run

Once installed, you just run it with `near` command:

```bash
$ near

? What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
> account     - Manage accounts
  tokens      - Manage token assets such as NEAR, FT, NFT
  staking     - Manage staking: view, add and withdraw stake
  contract    - Manage smart-contracts: deploy code, call functions
  transaction - Operate transactions
  config      - Manage connections in a configuration file (config.toml)
  extension   - Manage near CLI and extensions
[↑↓ to move, enter to select, type to filter]
```

The CLI interactively guides you through some pretty complex topics, helping you make informed decisions along the way.

## Shell Configuration for Command History

To enhance your experience with the NEAR CLI, you can configure your shell to integrate better with the near command. By adding the following functions to your shell configuration file, you ensure that commands executed via near are properly stored in your shell history and easily accessible via the arrow keys.

### Bash

Add the following function to your `~/.bashrc` file:

```bash
function near() {
    command near "$@"

    tmp_dir="${TMPDIR:-/tmp}"
    tmp_file="$tmp_dir/near-cli-rs-final-command.log"

    if [[ -f "$tmp_file" ]]; then
        final_command=$(<"$tmp_file")

        if [[ -n "$final_command" ]]; then
            history -s -- "$final_command"
        fi

        rm "$tmp_file"
    fi
}
```

### Zsh

Add the following function to your `~/.zshrc` file:

```zsh
function near() {
    command near "$@"

    tmp_dir="${TMPDIR:-/tmp}"
    tmp_file="$tmp_dir/near-cli-rs-final-command.log"

    if [[ -f "$tmp_file" ]]; then
        final_command=$(<"$tmp_file")

        if [[ -n "$final_command" ]]; then
            print -s -- "$final_command"
        fi

        rm "$tmp_file"
    fi
}
```

### Fish

Add the following function to your `~/.config/fish/config.fish` file:

```fish
function near
    command near $argv

    set tmp_dir (set -q TMPDIR; and echo $TMPDIR; or echo /tmp)
    set tmp_file "$tmp_dir/near-cli-rs-final-command.log"

    if test -f "$tmp_file"
        set -l final_command (cat "$tmp_file")

        if test -n "$final_command"
            set -l history_file (dirname (status --current-filename))/../fish_history

            if set -q XDG_DATA_HOME
                set history_file "$XDG_DATA_HOME/fish/fish_history"
            else if test -d "$HOME/.local/share/fish"
                set history_file "$HOME/.local/share/fish/fish_history"
            else
                set history_file "$HOME/.fish_history"
            end

            echo "$history_file"

            echo "- cmd: $final_command" >> $history_file
            echo "  when: "(date +%s) >> $history_file

            history --merge
        end

        rm "$tmp_file"
    end
end
```

> [!NOTE]
> For Fish shell, the function appends the command to the Fish history file and merges it to make it immediately accessible via the arrow keys.

### Explanation

These functions wrap the original near command and perform additional steps to read a command from a temporary log file, which is created by the NEAR CLI, and add it to your shell history. This allows you to easily access previous NEAR CLI commands using your shell's history mechanisms.

Steps performed by the functions:

- Run the original near command with all provided arguments.
- Check if the temporary log file exists.
- Read the command from the log file.
- If the command is not empty:
  - For Bash and Zsh: Add the command to the shell history.
  - For Fish: Append the command to the Fish history file and merge the history.
- Remove the temporary log file to prevent duplicate entries.

> [!IMPORTANT]
> Ensure that your NEAR CLI is configured to write the final command to the temporary log file at the specified location.
> Replace near with `cargo run --` in the functions if you are running the NEAR CLI via cargo locally.

## [Read more in English](docs/README.en.md)  
  - [Usage](docs/README.en.md#usage)
  - [Installation](docs/README.en.md#installation)
  - [User Guide](docs/README.en.md#user-guide)
  - [Config](docs/README.en.md#config)
  - [Building](docs/README.en.md#building)

## [Больше информации на русском языке (in Russian)](docs/README.ru.md)
  - [Применение](docs/README.ru.md#применение)
  - [Установка](docs/README.ru.md#установка)
  - [Инструкция](docs/README.ru.md#инструкция)
  - [Конфигурационный файл](docs/README.ru.md#конфигурационный-файл)
  - [Сборка](docs/README.ru.md#сборка)
