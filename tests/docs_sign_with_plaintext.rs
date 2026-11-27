//! Documented `sign-with-plaintext-private-key` examples must parse on the
//! current CLI. After near/near-cli-rs#468 the public key is derived from the
//! private key, and the private key is a positional argument rather than
//! `--signer-public-key` / `--signer-private-key`.

use std::path::Path;
use std::process::Command;

const DOC_PATHS: &[&str] = &["README.md", "docs/GUIDE.en.md", "docs/GUIDE.ru.md"];
const SIGN_CMD: &str = "sign-with-plaintext-private-key";
const REMOVED_FLAGS: &[&str] = &["--signer-public-key", "--signer-private-key"];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn near_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(option_env!("CARGO_BIN_EXE_near").unwrap_or("target/debug/near"))
}

/// Command examples inside fenced code blocks that use the plaintext signer.
fn documented_plaintext_signer_commands(markdown: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut rest = markdown;
    while let Some(start) = rest.find("```") {
        let after_ticks = &rest[start + 3..];
        let after_fence_header = match after_ticks.find('\n') {
            Some(i) => &after_ticks[i + 1..],
            None => break,
        };
        let Some(end) = after_fence_header.find("```") else {
            break;
        };
        let body = &after_fence_header[..end];
        rest = &after_fence_header[end + 3..];
        if !body.contains(SIGN_CMD) {
            continue;
        }
        commands.extend(near_commands_in_block(body));
    }
    commands
}

fn near_commands_in_block(body: &str) -> Vec<String> {
    let mut joined = Vec::new();
    let mut buf = String::new();
    for line in body.lines() {
        let raw = line.trim_end();
        let piece = if buf.is_empty() {
            raw.to_string()
        } else {
            format!("{} {}", buf.trim_end(), raw.trim_start())
        };
        if piece.ends_with('\\') {
            buf = piece[..piece.len() - 1].to_string();
            continue;
        }
        buf.clear();
        joined.push(piece);
    }
    if !buf.is_empty() {
        joined.push(buf);
    }

    joined
        .into_iter()
        .filter_map(|line| {
            let s = line.trim().trim_start_matches('$').trim();
            if s.starts_with("near ") || s.starts_with("./near") {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn split_shell_args(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some('\''), '\'') => quote = None,
            (Some('\''), other) => cur.push(other),
            (Some('"'), '"') => quote = None,
            (Some('"'), '\\') => {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    cur.push(next);
                }
            }
            (Some('"'), other) => cur.push(other),
            (None, '\'' | '"') => quote = Some(c),
            (None, c) if c.is_whitespace() => {
                if !cur.is_empty() {
                    args.push(std::mem::take(&mut cur));
                }
            }
            (None, other) => cur.push(other),
            (Some(_), other) => cur.push(other),
        }
    }
    if quote.is_some() {
        return Err(format!("unclosed quote in: {input}"));
    }
    if !cur.is_empty() {
        args.push(cur);
    }
    Ok(args)
}

fn collect_examples() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for rel in DOC_PATHS {
        let path = repo_root().join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for cmd in documented_plaintext_signer_commands(&text) {
            out.push((rel.to_string(), cmd));
        }
    }
    out
}

#[test]
fn documented_sign_with_plaintext_private_key_examples_parse() {
    let examples = collect_examples();
    assert!(
        !examples.is_empty(),
        "expected at least one {SIGN_CMD} example in README or the user guides"
    );

    let bin = near_bin();
    let mut failures = Vec::new();
    for (doc, cmd) in &examples {
        for flag in REMOVED_FLAGS {
            if cmd.contains(flag) {
                failures.push(format!(
                    "{doc}: example still uses removed flag {flag}:\n{cmd}"
                ));
            }
        }

        let mut args = split_shell_args(cmd).expect(cmd);
        if args
            .first()
            .is_some_and(|a| a == "near" || a == "./near" || a.ends_with("near.exe"))
        {
            args.remove(0);
        }
        // Env placeholders in the README CI example are not valid account IDs
        // or keys; replace them so clap can type-check the argv.
        args = args
            .into_iter()
            .map(|arg| substitute_placeholders(&arg))
            .collect();
        if !args.iter().any(|a| a == "--help" || a == "-h") {
            args.push("--help".to_string());
        }

        let output = Command::new(&bin)
            .args(&args)
            .output()
            .unwrap_or_else(|err| panic!("failed to run {}: {err}", bin.display()));
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let unexpected = text.to_lowercase().contains("unexpected argument")
            || text.to_lowercase().contains("unrecognized subcommand")
            || !output.status.success();
        if unexpected {
            failures.push(format!(
                "{doc}: command did not parse (status {:?}):\n{cmd}\n{text}",
                output.status.code()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "documented {SIGN_CMD} examples must parse on the current CLI:\n{}",
        failures.join("\n\n")
    );
}

fn substitute_placeholders(arg: &str) -> String {
    if arg == "$NEAR_CONTRACT_ACCOUNT_ID" || arg == "$NEAR_SIGNER_ACCOUNT_ID" {
        "example.near".to_string()
    } else if arg == "$NEAR_NETWORK_CONNECTION" {
        "mainnet".to_string()
    } else if arg == "$NEAR_SIGNER_ACCOUNT_PRIVATE_KEY" || arg.starts_with("$NEAR_SIGNER") {
        // Valid ed25519 secret used in the guides (and its matching public key).
        "ed25519:1aXaNaPNxU6Nwb4R1FxP9FzqFqhXwsx3nDS8PWv2jLxcX2ABEbKiGCPFKwEqQYzULWqiXLZDQX8oZYrhSLnDXFf"
            .to_string()
    } else {
        arg.to_string()
    }
}
