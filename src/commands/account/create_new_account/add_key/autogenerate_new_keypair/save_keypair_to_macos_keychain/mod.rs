use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SaveKeypairToMacosKeychain {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::super::SignerAccountId,
}

impl SaveKeypairToMacosKeychain {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        let network_config = self
            .sign_as
            .network_config
            .get_network_config(config.clone());
        crate::common::save_access_key_to_macos_keychain(
            network_config,
            key_pair_properties.clone(),
            account_properties
                .new_account_id
                .clone()
                .expect("Impossible to get contract_account_id!")
                .as_ref(),
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;
        let account_properties = super::super::super::AccountProperties {
            public_key: crate::types::public_key::PublicKey::from_str(
                &key_pair_properties.public_key_str,
            )?,
            ..account_properties
        };
        self.sign_as.process(config, account_properties).await
    }
}
