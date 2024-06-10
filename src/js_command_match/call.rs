#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `call` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct CallArgs {
    contract_account_id: String,
    method_name: String,
    #[clap(long, default_value_t = false)]
    base64: bool,
    args: String,
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: String,
    #[clap(long, default_value_t = 30_000_000_000_000)]
    gas: u64,
    #[clap(long, default_value = "0")]
    deposit: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl CallArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-transaction".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
        ];

        if self.base64 {
            command.push("base64-args".to_owned());
        } else {
            command.push("json-args".to_owned());            
        };

        
        command.push(self.args.to_owned());
        command.push("prepaid-gas".to_owned());
        command.push(format!("{} TeraGas", self.gas / 1_000_000_000_000));
        command.push("attached-deposit".to_owned());
        command.push(format!("{} NEAR", self.deposit));
        command.push("sign-as".to_owned());
        command.push(self.account_id.to_owned());
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
    fn call_with_json_args_testnet() {
        let call_args = CallArgs {
            contract_account_id: "contract.testnet".to_string(),
            method_name: "flip_coin".to_string(),
            base64: false,
            args: "{\"player_guess\": \"tail\"}".to_string(),
            account_id: "bob.testnet".to_string(),
            gas: 30000000000000,
            deposit: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CallArgs::to_cli_args(&call_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "contract call-function as-transaction {} flip_coin json-args {} prepaid-gas 30 TeraGas attached-deposit 0 NEAR sign-as {} network-config testnet sign-with-keychain send",
                call_args.contract_account_id,
                call_args.args,
                call_args.account_id,
            )
        )
    }

    #[test]
    fn call_with_json_args_mainnet() {
        let call_args = CallArgs {
            contract_account_id: "contract.testnet".to_string(),
            method_name: "flip_coin".to_string(),
            base64: false,
            args: "{\"player_guess\": \"tail\"}".to_string(),
            account_id: "bob.testnet".to_string(),
            gas: 30000000000000,
            deposit: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CallArgs::to_cli_args(&call_args, "mainnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "contract call-function as-transaction {} flip_coin json-args {} prepaid-gas 30 TeraGas attached-deposit 0 NEAR sign-as {} network-config mainnet sign-with-keychain send",
                call_args.contract_account_id,
                call_args.args,
                call_args.account_id,
            )
        )
    }

    #[test]
    fn call_with_base64_args_testnet() {
        let call_args = CallArgs {
            contract_account_id: "contract.testnet".to_string(),
            method_name: "flip_coin".to_string(),
            base64: true,
            args: "{\"player_guess\": \"tail\"}".to_string(),
            account_id: "bob.testnet".to_string(),
            gas: 30000000000000,
            deposit: "0".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CallArgs::to_cli_args(&call_args, "mainnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "contract call-function as-transaction {} flip_coin base64-args {} prepaid-gas 30 TeraGas attached-deposit 0 NEAR sign-as {} network-config mainnet sign-with-keychain send",
                call_args.contract_account_id,
                call_args.args,
                call_args.account_id,
            )
        )
    }
}