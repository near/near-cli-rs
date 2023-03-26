#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `set-api-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct SetApiKeyArgs {
    rpc_server: String,
    x_api_key: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl SetApiKeyArgs {
    pub fn to_cli_args(&self, network_name: String) -> color_eyre::eyre::Result<Vec<String>> {
        let config = crate::common::get_config_toml()?;
        let network_config = match config.network_connection.get(&network_name) {
            Some(network_config) => network_config,
            None => {
                return Ok(vec![
                    "config".to_owned(),
                    "add-connection".to_owned(),
                    "--network-name".to_owned(),
                    network_name.to_owned(),
                    "--connection-name".to_owned(),
                    network_name,
                    "--rpc-url".to_owned(),
                    self.rpc_server.to_owned(),
                    "--rpc-api-key".to_owned(),
                    self.x_api_key.to_owned(),
                ])
            }
        }
        .clone();
        let mut args = vec![
            "config".to_owned(),
            "add-connection".to_owned(),
            "--network-name".to_owned(),
            network_name.to_owned(),
            "--connection-name".to_owned(),
            network_name,
            "--rpc-url".to_owned(),
            self.rpc_server.to_owned(),
            "--wallet-url".to_owned(),
            network_config.wallet_url.to_string(),
            "--explorer-transaction-url".to_owned(),
            network_config.explorer_transaction_url.to_string(),
            "--rpc-api-key".to_owned(),
            self.x_api_key.to_owned(),
        ];
        if let Some(linkdrop_account_id) = network_config.linkdrop_account_id {
            args.push("--linkdrop-account-id".to_owned());
            args.push(linkdrop_account_id.to_string())
        }
        Ok(args)
    }
}
