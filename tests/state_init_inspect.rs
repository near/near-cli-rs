use std::path::{Path, PathBuf};
use std::process::Command;

/// Creates a minimal config pointing to a fake network.
/// Inspect commands never contact the network, so the URL doesn't matter.
fn setup_config(temp_dir: &Path) -> PathBuf {
    let config_home = temp_dir.to_path_buf();

    let near_cli_config_dir = if cfg!(target_os = "macos") {
        config_home.join("Library/Application Support/near-cli")
    } else {
        config_home.join("near-cli")
    };
    std::fs::create_dir_all(&near_cli_config_dir).unwrap();

    let credentials_dir = temp_dir.join("credentials");
    std::fs::create_dir_all(&credentials_dir).unwrap();
    std::fs::write(credentials_dir.join("ft_contracts.json"), "[]").unwrap();

    let config_content = format!(
        r#"version = "4"
credentials_home_dir = "{}"

[network_connection.sandbox]
network_name = "sandbox"
rpc_url = "http://localhost:3030/"
wallet_url = "http://localhost:3030/"
explorer_transaction_url = "http://localhost:3030/transactions/"
"#,
        credentials_dir.to_string_lossy()
    );
    std::fs::write(near_cli_config_dir.join("config.toml"), config_content).unwrap();

    config_home
}

fn near_cmd(config_home: &Path) -> Command {
    let mut cmd = Command::new("target/debug/near");
    cmd.env("XDG_CONFIG_HOME", config_home)
        .env("HOME", config_home)
        .env("APPDATA", config_home);
    cmd
}

/// A known all-zero code hash (valid bs58 for 32 zero bytes).
const ZERO_HASH: &str = "11111111111111111111111111111111";

/// True round-trip: build → borsh-base64 → re-parse via from-borsh-base64 → borsh-base64.
/// Both outputs must be identical.
#[test]
fn test_inspect_state_init_borsh_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = setup_config(temp_dir.path());

    // Step 1: build and serialise to borsh-base64
    let output1 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "borsh",
        ])
        .output()
        .unwrap();

    assert!(
        output1.status.success(),
        "step1 stderr: {}",
        String::from_utf8_lossy(&output1.stderr)
    );
    let borsh_b64 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    // Step 2: re-parse the borsh-base64 and serialise back to borsh-base64
    let output2 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-borsh-base64",
            &borsh_b64,
            "inspect",
            "state-init",
            "borsh",
        ])
        .output()
        .unwrap();

    assert!(
        output2.status.success(),
        "step2 stderr: {}",
        String::from_utf8_lossy(&output2.stderr)
    );
    let borsh_b64_roundtrip = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    assert_eq!(
        borsh_b64, borsh_b64_roundtrip,
        "borsh round-trip produced different output"
    );
}

#[test]
fn test_inspect_state_init_from_json_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = setup_config(temp_dir.path());

    // Step 1: build and serialise to JSON
    let output1 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output1.status.success(),);
    let json = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    // Step 2: re-parse from-json and serialise back to JSON
    let output2 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-json",
            &json,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output2.status.success(),);
    let json_roundtrip = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    assert_eq!(
        json, json_roundtrip,
        "from-json round-trip produced different output"
    );
}

#[test]
fn test_inspect_state_init_from_json_file_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = setup_config(temp_dir.path());

    let output1 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output1.status.success(),
        "step1 stderr: {}",
        String::from_utf8_lossy(&output1.stderr)
    );
    let json = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let json_file = temp_dir.path().join("state_init.json");
    std::fs::write(&json_file, &json).unwrap();

    let output2 = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-json-file",
            json_file.to_str().unwrap(),
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output2.status.success(),
        "step3 stderr: {}",
        String::from_utf8_lossy(&output2.stderr)
    );
    let json_roundtrip = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    assert_eq!(
        json, json_roundtrip,
        "from-json-file round-trip produced different output"
    );
}

#[test]
fn test_inspect_state_init_json_and_borsh_paths_consistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = setup_config(temp_dir.path());

    // Path A: build directly → JSON
    let output_a = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output_a.status.success(),
        "path A stderr: {}",
        String::from_utf8_lossy(&output_a.stderr)
    );
    let json_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();

    // Path B: build → borsh → from-borsh-base64 → JSON
    let borsh_output = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "borsh",
        ])
        .output()
        .unwrap();
    let borsh_b64 = String::from_utf8_lossy(&borsh_output.stdout)
        .trim()
        .to_string();

    let output_b = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-borsh-base64",
            &borsh_b64,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output_b.status.success(),
        "path B stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );
    let json_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    // Path C: build → JSON → from-json → JSON
    let output_c = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-json",
            &json_a,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output_c.status.success(),
        "path C stderr: {}",
        String::from_utf8_lossy(&output_c.stderr)
    );
    let json_c = String::from_utf8_lossy(&output_c.stdout).trim().to_string();

    assert_eq!(
        json_a, json_b,
        "JSON differs between direct and borsh paths"
    );
    assert_eq!(
        json_a, json_c,
        "JSON differs between direct and from-json paths"
    );
}

/// Cross-format round-trip: two input paths for the same StateInit must
/// produce identical JSON.
#[test]
fn test_inspect_state_init_json_consistent_across_input_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_home = setup_config(temp_dir.path());

    // Path A: build directly → JSON
    let output_a = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output_a.status.success(),
        "path A stderr: {}",
        String::from_utf8_lossy(&output_a.stderr)
    );
    let json_a = String::from_utf8_lossy(&output_a.stdout).to_string();

    // Path B: build → borsh → from-borsh-base64 → JSON
    let borsh_output = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "use-global-hash",
            ZERO_HASH,
            "data-from-json",
            r#"{"AAEC": "AwQF"}"#,
            "inspect",
            "state-init",
            "borsh",
        ])
        .output()
        .unwrap();
    let borsh_b64 = String::from_utf8_lossy(&borsh_output.stdout)
        .trim()
        .to_string();

    let output_b = near_cmd(&config_home)
        .args([
            "contract",
            "state-init",
            "from-borsh-base64",
            &borsh_b64,
            "inspect",
            "state-init",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output_b.status.success(),
        "path B stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );
    let json_b = String::from_utf8_lossy(&output_b.stdout).to_string();

    assert_eq!(json_a, json_b, "JSON output differs between input paths");
}
