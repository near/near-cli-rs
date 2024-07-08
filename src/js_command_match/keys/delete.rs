use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
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
    use super::*;
    use clap::Parser;

    #[test]
    fn delete_key_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let network_id = "testnet";

        for network_id_parameter_alias in NETWORK_ID_ALIASES {
            let delete_args = DeleteKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                &format!("--{network_id_parameter_alias}"),
                network_id,
            ]);
            let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account delete-keys {account_id} public-keys {access_key} network-config {network_id} sign-with-keychain send",
                )
            )
        }
    }

    #[test]
    fn delete_key_mainnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let network_id = "mainnet";

        let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[0]);
        let delete_args = DeleteKeyArgs::parse_from(&[
            "near",
            account_id,
            access_key,
            network_id_parameter_alias,
            network_id,
        ]);
        let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-keys {account_id} public-keys {access_key} network-config {network_id} sign-with-keychain send",
            )
        )
    }
}
