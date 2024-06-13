#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let command = vec![
            "account".to_owned(),
            "delete-keys".to_owned(),
            self.account_id.to_owned(),
            "public-keys".to_owned(),
            self.access_key.to_owned(),
            "network-config".to_owned(),
            network_id,
            "sign-with-legacy-keychain".to_owned(),
            "send".to_owned(),
        ];

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn delete_key_testnet() {
        let delete_args = DeleteKeyArgs {
            account_id: "bob.testnet".to_string(),
            access_key: "access_key_placeholder".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-keys {} {} public-keys network-config testnet sign-with-keychain send",
                delete_args.account_id,
                delete_args.access_key
            )
        )
    }

    #[test]
    fn delete_key_mainnet() {
        let delete_args = DeleteKeyArgs {
            account_id: "bob.testnet".to_string(),
            access_key: "access_key_placeholder".to_string(),
            network_id: Some("mainnet".to_owned()),
            _unknown_args: [].to_vec(),
        };
        let result = DeleteKeyArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-keys {} {} public-keys network-config mainnet sign-with-keychain send",
                delete_args.account_id,
                delete_args.access_key
            )
        )
    }
}
