use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct AddAccessWithLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

impl interactive_clap::FromCli for AddAccessWithLedger {
    type FromCliContext = crate::commands::account::create_account::CreateAccountContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<AddAccessWithLedger as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let seed_phrase_hd_path = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path.clone())
        {
            Some(hd_path) => hd_path,
            None => Self::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let sign_as = super::super::sign_as::SignerAccountId::from_cli(
            optional_clap_variant.and_then(|clap_variant| {
                clap_variant.sign_as.map(
                    |ClapNamedArgSignerAccountIdForAddAccessWithLedger::SignAs(cli_signer)| {
                        cli_signer
                    },
                )
            }),
            context,
        )?;
        let sign_as = if let Some(value) = sign_as {
            value
        } else {
            return Ok(None);
        };

        Ok(Some(Self {
            seed_phrase_hd_path,
            sign_as,
        }))
    }
}

impl AddAccessWithLedger {
    pub fn input_seed_phrase_hd_path() -> crate::types::slip10::BIP32Path {
        crate::types::slip10::BIP32Path::from_str(
            &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                .with_initial_value("44'/397'/0'/0'/1'")
                .prompt()
                .unwrap(),
        )
        .unwrap()
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        let public_key = near_ledger::get_public_key(self.seed_phrase_hd_path.clone().into())
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            })?;
        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            public_key.to_bytes(),
        ));
        let account_properties = super::super::AccountProperties {
            public_key,
            ..account_properties
        };
        let storage_properties = None;
        self.sign_as
            .process(config, account_properties, storage_properties)
            .await
    }
}
