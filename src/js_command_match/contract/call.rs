use near_gas::NearGas;

use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
    USE_ACCOUNT_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct CallArgs {
    contract_name: String,
    method_name: String,
    #[arg(default_value = "{}")]
    args: String,
    #[arg(long, aliases = USE_ACCOUNT_ALIASES)]
    use_account: String,
    #[arg(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[arg(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[arg(long, default_value_t = 30_000_000_000_000)]
    gas: u64,
    #[arg(long, default_value = "0")]
    deposit: String,
    #[arg(long, default_value = "0", conflicts_with = "deposit", aliases = ["depositYocto"])]
    deposit_yocto: String,
    #[arg(long, default_value_t = false)]
    base64: bool,
    #[arg(long, aliases = ["privateKey"])]
    private_key: Option<String>,
    #[arg(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
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
            command.push(self.ledger_path.to_owned());
        } else if let Some(private_key) = &self.private_key {
            command.push("sign-with-plaintext-private-key".to_string());
            command.push(private_key.to_string());
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
    fn call() {
        let json_args = "{\"player_guess\": \"tail\"}";
        let base64_args = "eyJwbGF5ZXJfZ3Vlc3MiOiAidGFpbCJ9";

        for (input, expected_output) in [
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[0]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[1]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[2]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[3]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[4]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --{} bob.testnet",
                    USE_ACCOUNT_ALIASES[5]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin {base64_args} --useAccount bob.testnet --base64"
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin base64-args {base64_args} prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[0]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[1]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[2]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{}",
                    SIGN_WITH_LEDGER_ALIASES[3]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\"",
                    LEDGER_PATH_ALIASES[0]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\"",
                    LEDGER_PATH_ALIASES[1]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{} mainnet",
                    NETWORK_ID_ALIASES[0]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config mainnet sign-with-keychain send"
                ),
            ),
            (
                format!(
                    "near call contract.testnet flip_coin '{json_args}' --useAccount bob.testnet --{} mainnet",
                    NETWORK_ID_ALIASES[1]
                ),
                format!(
                    "contract call-function as-transaction contract.testnet flip_coin json-args '{json_args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' sign-as bob.testnet network-config mainnet sign-with-keychain send"
                ),
            ),
        ] {
            let input_cmd =
                shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::Call(call_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("Call command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(CallArgs::to_cli_args(&call_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
