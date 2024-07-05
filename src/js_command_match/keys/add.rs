use crate::js_command_match::constants::{
    CONTRACT_ID_ALIASES, DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, METHOD_NAMES_ALIASES,
    NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct AddKeyArgs {
    account_id: String,
    public_key: String,
    #[clap(long, aliases = CONTRACT_ID_ALIASES, default_value = None)]
    contract_id: Option<String>,
    #[clap(long, aliases = METHOD_NAMES_ALIASES, requires = "contract_id", default_value="", value_delimiter = ',', num_args = 0..)]
    method_names: Vec<String>,
    #[clap(long, default_value = "0")]
    allowance: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = Some(DEFAULT_SEED_PHRASE_PATH))]
    ledger_path: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
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

        if let Some(contract_id) = self.contract_id.as_deref() {
            let allowance = if self.allowance != "0" {
                format!("{} NEAR", self.allowance)
            } else {
                "unlimited".to_string()
            };

            command.push("grant-function-call-access".to_string());
            command.push("--allowance".to_string());
            command.push(allowance);
            command.push("--contract-account-id".to_string());
            command.push(contract_id.to_owned());
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
            command.push(self.ledger_path.to_owned().unwrap_or_default());
        } else {
            command.push("sign-with-keychain".to_string());
        }
        command.push("send".to_string());

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn add_function_call_access_key_for_all_methods_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let contract_id = "example.testnet";

        for i in 0..CONTRACT_ID_ALIASES.len() {
            let contract_id_parameter_alias = &format!("--{}", &CONTRACT_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                contract_id_parameter_alias,
                contract_id,
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-function-call-access --allowance unlimited --contract-account-id {} --function-names  use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, contract_id, access_key)
            )
        }
    }

    #[test]
    fn add_function_call_access_key_for_some_methods_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let contract_id = "example.testnet";
        let method_names = "get,set";

        for i in 0..METHOD_NAMES_ALIASES.len() {
            let method_names_parameter_alias = &format!("--{}", &METHOD_NAMES_ALIASES[i]);
            let contract_id_parameter_alias = &format!("--{}", &CONTRACT_ID_ALIASES[0]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                contract_id_parameter_alias,
                contract_id,
                method_names_parameter_alias,
                method_names,
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {account_id} grant-function-call-access --allowance unlimited --contract-account-id {contract_id} --function-names {method_names} use-manually-provided-public-key {access_key} network-config testnet sign-with-keychain send")
            )
        }
    }

    #[test]
    fn add_full_access_key_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                network_id_parameter_alias,
                "testnet",
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, access_key)
            )
        }
    }

    #[test]
    fn add_full_access_key_mainnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let network_id = "mainnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                network_id_parameter_alias,
                network_id,
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config {} sign-with-keychain send", account_id, access_key, network_id)
            )
        }
    }
}
