use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "account".to_string(),
            "delete-keys".to_string(),
            self.account_id.to_owned(),
            "public-keys".to_string(),
            self.access_key.to_owned(),
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
    fn delete_key_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let network_id = "testnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let delete_args = DeleteKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                network_id_parameter_alias,
                network_id
            ]);
            let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account delete-keys {} public-keys {} network-config {} sign-with-keychain send",
                    account_id,
                    access_key,
                    network_id
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
            network_id
        ]);
        let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-keys {} public-keys {} network-config {} sign-with-keychain send",
                account_id,
                access_key,
                network_id
            )
        )
    }
}
