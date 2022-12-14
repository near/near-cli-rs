#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct LoginFromPrivateKey {
    /// Enter your private (secret) key
    private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network::Network,
}

impl LoginFromPrivateKey {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let secret_key: near_crypto::SecretKey = self.private_key.clone().into();
        let public_key = secret_key.public_key();
        let key_pair_properties_buf = serde_json::json!({
            "public_key": public_key.to_string(),
            "private_key": secret_key.to_string(),
        })
        .to_string();
        let error_message = "\nIt is currently not possible to verify the account access key.\nYou may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n";
        super::login(
            network_config,
            config.credentials_home_dir,
            &key_pair_properties_buf,
            &public_key.to_string(),
            error_message,
        )
        .await
    }
}
