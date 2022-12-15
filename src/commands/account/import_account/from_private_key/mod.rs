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

#[derive(Debug, serde::Serialize)]
struct KeyPairProperties {
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}

impl LoginFromPrivateKey {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let private_key: near_crypto::SecretKey = self.private_key.clone().into();
        let public_key = private_key.public_key();
        let key_pair_properties = KeyPairProperties {
            public_key: public_key.clone(),
            private_key
        };
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties).unwrap();
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
