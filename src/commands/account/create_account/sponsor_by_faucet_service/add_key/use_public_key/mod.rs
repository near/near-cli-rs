#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AddAccessKeyAction {
    ///Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: super::super::network::Network,
}

impl AddAccessKeyAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        let account_properties = super::super::super::AccountProperties {
            public_key: self.public_key.clone().into(),
            ..account_properties
        };
        let storage_message = None;
        self.network_config
            .process(config, account_properties, storage_message)
            .await
    }
}
