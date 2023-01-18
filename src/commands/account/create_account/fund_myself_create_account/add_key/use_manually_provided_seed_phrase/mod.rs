use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
pub struct AddAccessWithSeedPhraseAction {
    ///Enter the seed-phrase for this sub-account
    master_seed_phrase: String,
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

impl AddAccessWithSeedPhraseAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        // This is the HD path that is used in NEAR Wallet for plaintext seed phrase generation and, subsequently, for account recovery by a seed phrase.
        let near_wallet_seed_phrase_hd_path_default =
            slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            near_wallet_seed_phrase_hd_path_default,
            &self.master_seed_phrase,
        )?;
        let account_properties = super::super::super::AccountProperties {
            public_key,
            ..account_properties
        };
        let storage_properties = None;
        self.sign_as
            .process(config, account_properties, storage_properties)
            .await
    }
}
