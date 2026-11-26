#![cfg(unix)]

use std::process::Command;

fn near_error(args: &[&str]) -> String {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_near"))
        .args(args)
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("PATH", temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    String::from_utf8(output.stderr).unwrap()
}

#[test]
fn native_mode_flags_do_not_hide_invalid_transaction_values() {
    for flag in ["--quiet", "--offline", "--teach-me"] {
        let error = near_error(&[flag, "transaction", "print-transaction", "signed", "###"]);
        assert!(
            error.contains("base64 transaction sequence is invalid"),
            "{error}"
        );
    }
}

#[test]
fn unknown_command_with_a_mode_flag_uses_the_extension_diagnostic() {
    let error = near_error(&["--quiet", "unknown-subcommand-for-test"]);
    assert!(
        error.contains("unknown-subcommand-for-test command or near-unknown-subcommand-for-test extension does not exist"),
        "{error}"
    );
}
