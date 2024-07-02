use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteArgs {
    account_id: String,
    beneficiary_id: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "account".to_string(),
            "delete-account".to_string(),
            self.account_id.to_owned(),
            "beneficiary".to_string(),
            self.beneficiary_id.to_owned(),
            "network-config".to_string(),
            network_id,
            "sign-with-keychain".to_string(),
            "send".to_string(),
        ];

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
 
    #[test]
    fn delete_testnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";
        let network_id = "testnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let delete_args = DeleteArgs::parse_from(&[
                "near",
                account_id,
                beneficiary_id,
                network_id_parameter_alias,
                network_id
            ]);
            let result = DeleteArgs::to_cli_args(&delete_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account delete-account {} beneficiary {} network-config {} sign-with-keychain send",
                    account_id,
                    beneficiary_id,
                    network_id
                )
            );
        }
    }

    #[test]
    fn delete_mainnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";
        let network_id = "mainnet";

        let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[0]);
        let delete_args = DeleteArgs::parse_from(&[
            "near",
            account_id,
            beneficiary_id,
            network_id_parameter_alias,
            network_id
        ]);
        let result = DeleteArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-account {} beneficiary {} network-config {} sign-with-keychain send",
                account_id,
                beneficiary_id,
                network_id
            )
        );
    }
}