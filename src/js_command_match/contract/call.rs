use near_gas::NearGas;

use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
    USE_ACCOUNT_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct CallArgs {
    contract_name: String,
    method_name: String,
    #[clap(default_value = "")]
    args: String,
    #[clap(long, aliases = USE_ACCOUNT_ALIASES)]
    use_account: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = Some(DEFAULT_SEED_PHRASE_PATH))]
    ledger_path: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(long, default_value_t = 30_000_000_000_000)]
    gas: u64,
    #[clap(long, default_value = "0")]
    deposit: String,
    #[clap(long, default_value = "0", conflicts_with = "deposit", aliases = ["depositYocto"])]
    deposit_yocto: String,
    #[clap(long, default_value_t = false)]
    base64: bool,
    #[clap(long, aliases = ["privateKey"], default_value=None)]
    private_key: Option<String>,
}

impl CallArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "contract".to_string(),
            "call-function".to_string(),
            "as-transaction".to_string(),
            self.contract_name.to_owned(),
            self.method_name.to_owned(),
        ];

        if self.base64 {
            command.push("base64-args".to_string());
        } else {
            command.push("json-args".to_string());
        };

        command.push(self.args.to_owned());
        command.push("prepaid-gas".to_string());
        command.push(format!("{} Tgas", NearGas::from_gas(self.gas).as_tgas()));
        command.push("attached-deposit".to_string());

        if self.deposit_yocto != "0" {
            command.push(format!("{} yoctonear", self.deposit_yocto));
        } else {
            command.push(format!("{} NEAR", self.deposit));
        }

        command.push("sign-as".to_string());
        command.push(self.use_account.to_owned());
        command.push("network-config".to_string());
        command.push(network_id);

        if self.sign_with_ledger {
            command.push("sign-with-ledger".to_string());
            command.push("--seed-phrase-hd-path".to_string());
            command.push(self.ledger_path.to_owned().unwrap_or_default());
        }

        if self.private_key.is_some() {
            command.push("sign-with-plaintext-private-key".to_string());
            command.push(self.private_key.clone().unwrap());
        }

        if !self.sign_with_ledger && !self.private_key.is_some() {
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
    fn call_with_json_args_testnet() {
        let contract_account_id = "contract.testnet";
        let method_name = "flip_coin";
        let args = "{\"player_guess\": \"tail\"}";
        let account_id = "bob.testnet";

        let call_args = CallArgs::parse_from(&[
            "near",
            contract_account_id,
            method_name,
            args,
            "--useAccount",
            account_id,
        ]);
        let result = CallArgs::to_cli_args(&call_args, "testnet".to_string());
        assert_eq!(
                result.join(" "),
                format!(
                    "contract call-function as-transaction {} flip_coin json-args {} prepaid-gas 30000000000000 gas attached-deposit 0 NEAR sign-as {} network-config testnet sign-with-keychain send",
                    contract_account_id,
                    args,
                    account_id,
                )
            )
    }

    #[test]
    fn call_with_json_args_mainnet() {
        let contract_account_id = "contract.testnet";
        let method_name = "flip_coin";
        let args = "{\"player_guess\": \"tail\"}";
        let account_id = "bob.testnet";
        let network_id = "mainnet";

        let call_args = CallArgs::parse_from(&[
            "near",
            contract_account_id,
            method_name,
            args,
            "--useAccount",
            account_id,
            "--networkId",
            network_id,
        ]);
        let result = CallArgs::to_cli_args(&call_args, "testnet".to_string());
        assert_eq!(
                result.join(" "),
                format!(
                    "contract call-function as-transaction {} flip_coin json-args {} prepaid-gas 30000000000000 gas attached-deposit 0 NEAR sign-as {} network-config {} sign-with-keychain send",
                    contract_account_id,
                    args,
                    account_id,
                    network_id,
                )
            )
    }

    #[test]
    fn call_with_base64_args_testnet() {
        let contract_account_id = "contract.testnet";
        let method_name = "flip_coin";
        let args = "{\"player_guess\": \"tail\"}";
        let account_id = "bob.testnet";

        let call_args = CallArgs::parse_from(&[
            "near",
            contract_account_id,
            method_name,
            args,
            "--useAccount",
            account_id,
            "--base64",
        ]);
        let result = CallArgs::to_cli_args(&call_args, "testnet".to_string());
        assert_eq!(
                result.join(" "),
                format!(
                    "contract call-function as-transaction {} flip_coin base64-args {} prepaid-gas 30000000000000 gas attached-deposit 0 NEAR sign-as {} network-config testnet sign-with-keychain send",
                    contract_account_id,
                    args,
                    account_id,
                )
            )
    }
}
