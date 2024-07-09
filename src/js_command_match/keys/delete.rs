use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl DeleteKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "account".to_string(),
            "delete-keys".to_string(),
            self.account_id.to_owned(),
            "public-keys".to_string(),
            self.access_key.to_owned(),
            "network-config".to_string(),
            network_id,
        ];

        if self.sign_with_ledger {
            command.push("sign-with-ledger".to_string());
            command.push("--seed-phrase-hd-path".to_string());
            command.push(self.ledger_path.to_owned());
        } else {
            command.push("sign-with-keychain".to_string());
        }

        command.push("send".to_string());

        command
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn delete_key() {
        for (input, expected_output) in [
            (
                "near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq".to_string(),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{}", SIGN_WITH_LEDGER_ALIASES[0]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{}", SIGN_WITH_LEDGER_ALIASES[1]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --signWithLedger --{} \"44'/397'/0'/0'/2'\"", LEDGER_PATH_ALIASES[0]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --signWithLedger --{} \"44'/397'/0'/0'/2'\"", LEDGER_PATH_ALIASES[1]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} testnet", NETWORK_ID_ALIASES[0]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send".to_string()
            ),
            (
                format!("near delete-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} mainnet", NETWORK_ID_ALIASES[1]),
                "account delete-keys bob.testnet public-keys ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config mainnet sign-with-keychain send".to_string()
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::DeleteKey(delete_key_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("DeleteKey command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(DeleteKeyArgs::to_cli_args(&delete_key_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
