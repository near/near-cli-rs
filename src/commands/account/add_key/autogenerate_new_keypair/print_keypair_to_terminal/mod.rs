#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct PrintKeypairToTerminal {
    #[interactive_clap(named_arg)]
    ///Select network
    network: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl PrintKeypairToTerminal {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        println!(
            "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
            key_pair_properties.master_seed_phrase,
            key_pair_properties.seed_phrase_hd_path,
            key_pair_properties.implicit_account_id,
            key_pair_properties.public_key_str,
            key_pair_properties.secret_keypair_str,
        );
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
