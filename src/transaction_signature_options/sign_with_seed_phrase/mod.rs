use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignSeedPhrase {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    submit: Option<super::Submit>,
}

impl interactive_clap::FromCli for SignSeedPhrase {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignSeedPhrase as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let master_seed_phrase: String = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.master_seed_phrase.as_ref())
        {
            Some(master_seed_phrase) => master_seed_phrase.clone(),
            None => Self::input_master_seed_phrase(&context)?,
        };
        let seed_phrase_hd_path: crate::types::slip10::BIP32Path = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path.as_ref())
        {
            Some(seed_phrase_hd_path) => seed_phrase_hd_path.clone(),
            None => Self::input_seed_phrase_hd_path(&context)?,
        };
        let submit: Option<super::Submit> =
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            master_seed_phrase,
            seed_phrase_hd_path,
            submit,
        }))
    }
}

impl SignSeedPhrase {
    fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::slip10::BIP32Path> {
        Ok(
            inquire::CustomType::new("Enter seed phrase HD Path [if not sure, keep the default]")
                .with_default(crate::types::slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap())
                .prompt()?,
        )
    }

    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            self.seed_phrase_hd_path.clone(),
            self.master_seed_phrase.clone(),
        )?;
        let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
            signer_public_key: crate::types::public_key::PublicKey::from_str(
                &key_pair_properties.public_key_str,
            )?,
            signer_private_key: crate::types::secret_key::SecretKey::from_str(
                &key_pair_properties.secret_keypair_str,
            )?,
            nonce: None,
            block_hash: None,
            submit: self.submit.clone(),
        };
        sign_with_private_key
            .process(prepopulated_unsigned_transaction, network_config)
            .await
    }
}
