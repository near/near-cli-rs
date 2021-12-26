use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    pub permission: super::add_access_key::AccessKeyPermission,
}

impl GenerateKeypair {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;

        crate::common::save_access_key_to_keychain(
            network_connection_config.clone(),
            key_pair_properties.clone(),
            &prepopulated_unsigned_transaction.signer_id.to_string(),
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;

        match self.permission {
            super::add_access_key::AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                    )
                    .await
            }
            super::add_access_key::AccessKeyPermission::GrantFunctionCallAccess(
                function_call_type,
            ) => {
                function_call_type
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                    )
                    .await
            }
        }
    }
}
