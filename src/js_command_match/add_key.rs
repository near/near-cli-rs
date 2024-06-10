#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `add-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = ["contractId"], default_value = None)]
    contract_id: Option<String>,
    #[clap(long, aliases = ["methodNames"], requires = "contract_id", value_delimiter = ',', num_args = 1..)]
    method_names: Vec<String>,
    #[clap(long, default_value = "0")]
    allowance: String,
    #[clap(long, aliases = ["networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "account".to_owned(),
            "add-key".to_owned(),
            self.account_id.to_owned(),
        ];

        if let Some(contract_id) = self.contract_id.as_deref() {
            command.push("grant-function-call-access".to_owned());
            command.push("--allowance".to_owned());
            command.push(format!("{} NEAR", self.allowance));
            command.push("--receiver-account-id".to_owned());
            command.push(contract_id.to_owned());
            command.push("--method-names".to_owned());
            command.push(self.method_names.join(","));
        } else {
            command.push("grant-full-access".to_owned());
        }
          
        command.push("use-manually-provided-public-key".to_owned());
        command.push(self.access_key.to_owned());
        command.push("network-config".to_owned());
        command.push(network_id);
        command.push("sign-with-keychain".to_owned());
        command.push("send".to_owned());

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_function_call_access_key_testnet() {
        let account_id = "bob.testnet".to_string();
        let access_key = "access_key_placeholder".to_string();
        let contract_id = "example.testnet".to_string();

        let add_key_args = AddKeyArgs {
            account_id: account_id.clone(),
            access_key: access_key.clone(),
            contract_id: Some(contract_id.clone()),
            method_names: [].to_vec(),
            allowance: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account add-key {} grant-function-call-access --allowance 0 NEAR --receiver-account-id {} --method-names  use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, contract_id, access_key)
        )
    }

    #[test]
    fn add_full_access_key_testnet() {
        let account_id = "bob.testnet".to_string();
        let access_key = "access_key_placeholder".to_string();

        let add_key_args = AddKeyArgs {
            account_id: account_id.clone(),
            access_key: access_key.clone(),
            contract_id: None,
            method_names: [].to_vec(),
            allowance: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, access_key)
        )
    }

    #[test]
    fn add_full_access_key_mainnet() {
        let account_id = "bob.testnet".to_string();
        let access_key = "access_key_placeholder".to_string();

        let add_key_args = AddKeyArgs {
            account_id: account_id.clone(),
            access_key: access_key.clone(),
            contract_id: None,
            method_names: [].to_vec(),
            allowance: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = AddKeyArgs::to_cli_args(&add_key_args, "mainnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config mainnet sign-with-keychain send", account_id, access_key)
        )
    }
}