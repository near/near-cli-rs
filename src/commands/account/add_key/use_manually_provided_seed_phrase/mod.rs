use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AddAccessWithSeedPhraseAction {
    ///Enter the seed-phrase for this sub-account
    master_seed_phrase: String,
    #[interactive_clap(named_arg)]
    ///Select network
    network: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl AddAccessWithSeedPhraseAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        let seed_phrase_hd_path_default = slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            seed_phrase_hd_path_default,
            &self.master_seed_phrase,
        )?;
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };

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
