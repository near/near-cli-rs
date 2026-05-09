use color_eyre::eyre::WrapErr;
use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignAccessKeyFileContext)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file (path/to/access-key-file.json)?
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_height: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignAccessKeyFileContext(super::SubmitContext);

impl SignAccessKeyFileContext {
    #[tracing::instrument(
        name = "Signing the transaction using the account access key file ...",
        skip_all
    )]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignAccessKeyFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Signing the transaction using the account access key file ...");

        let data =
            std::fs::read_to_string(&scope.file_path).wrap_err("Access key file not found!")?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &scope.file_path))?;

        let nk_public_key = account_json.public_key.clone();
        let nk_secret_key = account_json.private_key.clone();

        let (nonce, block_hash, block_height) = super::resolve_nonce_and_block(
            &previous_context.network_config,
            &previous_context.prepopulated_transaction.signer_id,
            &nk_public_key,
            previous_context.global_context.offline,
            scope.nonce,
            scope.block_hash,
            scope.block_height,
        )?;

        Ok(Self(super::sign_transaction_with_secret_key(
            nk_public_key,
            nk_secret_key,
            previous_context,
            nonce,
            block_hash,
            block_height,
            scope.meta_transaction_valid_for,
        )?))
    }
}

impl From<SignAccessKeyFileContext> for super::SubmitContext {
    fn from(item: SignAccessKeyFileContext) -> Self {
        item.0
    }
}

impl SignAccessKeyFile {
    fn input_nonce(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter a nonce for the access key:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_hash(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::crypto_hash::CryptoHash>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<crate::types::crypto_hash::CryptoHash>::new(
                    "Enter recent block hash:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_height(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter recent block height:").prompt()?,
            ));
        }
        Ok(None)
    }
}
