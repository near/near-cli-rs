use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct PrintKeypairToTerminal {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::super::SignerAccountId,
}

impl PrintKeypairToTerminal {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        println!(
            "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
            key_pair_properties.master_seed_phrase,
            key_pair_properties.seed_phrase_hd_path,
            key_pair_properties.implicit_account_id,
            key_pair_properties.public_key_str,
            key_pair_properties.secret_keypair_str,
        );
        let account_properties = super::super::super::AccountProperties {
            public_key: crate::types::public_key::PublicKey::from_str(
                &key_pair_properties.public_key_str,
            )?,
            ..account_properties
        };
        self.sign_as.process(config, account_properties).await
    }
}
