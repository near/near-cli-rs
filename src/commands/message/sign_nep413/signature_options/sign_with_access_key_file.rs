use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::FinalSignNep413Context)]
#[interactive_clap(output_context = SignAccessKeyFileContext)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file?
    file_path: crate::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SignAccessKeyFileContext;

impl SignAccessKeyFileContext {
    pub fn from_previous_context(
        previous_context: super::super::FinalSignNep413Context,
        scope: &<SignAccessKeyFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data =
            std::fs::read_to_string(&scope.file_path).wrap_err("Access key file not found!")?;
        let account_json: crate::transaction_signature_options::AccountKeyPair =
            serde_json::from_str(&data).wrap_err_with(|| {
                format!("Error reading data from file: {:?}", &scope.file_path)
            })?;

        let signature = super::super::sign_nep413_payload(
            &previous_context.payload,
            &account_json.private_key,
        )?;

        let signed_message = super::super::SignedMessage {
            account_id: previous_context.signer_id.to_string(),
            public_key: account_json.public_key.to_string(),
            signature: signature.to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&signed_message)?);
        Ok(Self)
    }
}
