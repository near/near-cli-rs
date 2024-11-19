# Shell Configuration for Command History

To enhance your experience with the NEAR CLI, you can configure your shell to integrate better with the near command. By adding the following functions to your shell configuration file, you ensure that commands executed via near are properly stored in your shell history and easily accessible via the arrow keys.

## Bash

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

## Zsh

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

## Fish

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

## Explanation

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
