#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AddAccessKeyAction {
    ///Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network::Network,
}

impl AddAccessKeyAction {
    pub fn get_public_key(&self) -> crate::types::public_key::PublicKey {
        self.public_key.clone()
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        self.network_config.get_network_config(config)
    }
}
