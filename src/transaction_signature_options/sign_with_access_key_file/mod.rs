#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file (path/to/access-key-file.json)?
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

// impl interactive_clap::FromCli for SignAccessKeyFile {
//     type FromCliContext = crate::commands::TransactionContext;
//     type FromCliError = color_eyre::eyre::Error;

//     fn from_cli(
//         optional_clap_variant: Option<<SignAccessKeyFile as interactive_clap::ToCli>::CliVariant>,
//         context: Self::FromCliContext,
//     ) -> Result<Option<Self>, Self::FromCliError>
//     where
//         Self: Sized + interactive_clap::ToCli,
//     {
//         let file_path: crate::types::path_buf::PathBuf = match optional_clap_variant
//             .clone()
//             .and_then(|clap_variant| clap_variant.file_path)
//         {
//             Some(cli_file_path) => cli_file_path,
//             None => Self::input_file_path(&context)?,
//         };
//         let optional_submit = super::Submit::from_cli(
//             optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
//             context,
//         )?;
//         let submit = if let Some(submit) = optional_submit {
//             submit
//         } else {
//             return Ok(None);
//         };
//         Ok(Some(Self { file_path, submit }))
//     }
// }

impl SignAccessKeyFile {
    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let data = std::fs::read_to_string(&self.file_path).map_err(|err| {
            color_eyre::Report::msg(format!("Access key file not found! Error: {}", err))
        })?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;
        let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
            signer_public_key: crate::types::public_key::PublicKey(account_json.public_key),
            signer_private_key: crate::types::secret_key::SecretKey(account_json.private_key),
            nonce: None,
            block_hash: None,
            submit: self.submit.clone(),
        };
        sign_with_private_key
            .process(prepopulated_unsigned_transaction, network_config)
            .await
    }
}
