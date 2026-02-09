use crate::js_command_match::constants::{
    CONTRACT_ID_ALIASES, DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, METHOD_NAMES_ALIASES,
    NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct AddKeyArgs {
    account_id: String,
    public_key: String,
    #[arg(long, aliases = CONTRACT_ID_ALIASES)]
    contract_id: Option<String>,
    #[arg(long, aliases = METHOD_NAMES_ALIASES, requires = "contract_id", default_value="", value_delimiter = ',', num_args = 0..)]
    method_names: Vec<String>,
    #[arg(long, default_value = "0")]
    allowance: String,
    #[arg(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[arg(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[arg(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl AddKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "account".to_string(),
            "add-key".to_string(),
            self.account_id.to_owned(),
        ];

        if let Some(contract_id) = &self.contract_id {
            let allowance = if self.allowance != "0" {
                format!("{} NEAR", self.allowance)
            } else {
                "unlimited".to_string()
            };

            command.push("grant-function-call-access".to_string());
            command.push("--allowance".to_string());
            command.push(allowance);
            command.push("--contract-account-id".to_string());
            command.push(contract_id.to_string());
            command.push("--function-names".to_string());
            command.push(self.method_names.join(","));
        } else {
            command.push("grant-full-access".to_string());
        }

        command.push("use-manually-provided-public-key".to_string());
        command.push(self.public_key.to_owned());
        command.push("network-config".to_string());
        command.push(network_id);

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
    fn add_key() {
        for (input, expected_output) in [
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} contract.testnet",
                    CONTRACT_ID_ALIASES[0]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} contract.testnet",
                    CONTRACT_ID_ALIASES[1]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{} get,set",
                    METHOD_NAMES_ALIASES[0]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names get,set use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{} get,set",
                    METHOD_NAMES_ALIASES[1]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names get,set use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[0]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[1]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[2]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[3]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\"",
                    LEDGER_PATH_ALIASES[0]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --contractId contract.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\"",
                    LEDGER_PATH_ALIASES[1]
                ),
                "account add-key bob.testnet grant-function-call-access --allowance unlimited --contract-account-id contract.testnet --function-names '' use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} testnet",
                    NETWORK_ID_ALIASES[0]
                ),
                "account add-key bob.testnet grant-full-access use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config testnet sign-with-keychain send",
            ),
            (
                format!(
                    "near add-key bob.testnet ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq --{} mainnet",
                    NETWORK_ID_ALIASES[1]
                ),
                "account add-key bob.testnet grant-full-access use-manually-provided-public-key ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq network-config mainnet sign-with-keychain send",
            ),
        ] {
            let input_cmd =
                shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::AddKey(add_key_args) = JsCmd::parse_from(&input_cmd) else {
                panic!(
                    "AddKey command was expected, but something else was parsed out from {input}"
                );
            };
            assert_eq!(
                shell_words::join(AddKeyArgs::to_cli_args(
                    &add_key_args,
                    "testnet".to_string()
                )),
                expected_output
            );
        }
    }
}
