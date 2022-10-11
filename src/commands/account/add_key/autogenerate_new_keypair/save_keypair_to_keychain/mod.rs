#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SaveKeypairToKeychain {
    #[interactive_clap(named_arg)]
    ///Select network
    network: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SaveKeypairToKeychain {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let network_config = self.network.get_network_config(config.clone());
        crate::common::save_access_key_to_keychain(
            network_config,
            config.credentials_home_dir.clone(),
            key_pair_properties,
            &prepopulated_unsigned_transaction.receiver_id,
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;
        match self.network.get_sign_option() {
            crate::transaction_signature_options::SignWith::SignWithPlaintextPrivateKey(
                sign_private_key,
            ) => {
                sign_private_key
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config),
                    )
                    .await
            }
            crate::transaction_signature_options::SignWith::SignWithKeychain(sign_keychain) => {
                sign_keychain
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config.clone()),
                        config.credentials_home_dir,
                    )
                    .await
            }
            #[cfg(feature = "ledger")]
            crate::transaction_signature_options::SignWith::SignWithLedger(sign_ledger) => {
                sign_ledger
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config),
                    )
                    .await
            }
        }
    }
}
