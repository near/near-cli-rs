use std::io::Write;

use color_eyre::eyre::Context;
use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SignLaterContext)]
#[interactive_clap(output_context = SaveToFileContext)]
pub struct SaveToFile {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the location of the file to save the unsigned transaction (path/to/signed-transaction-info.json)?
    file_path: crate::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SaveToFileContext;

impl SaveToFileContext {
    pub fn from_previous_context(
        previous_context: super::SignLaterContext,
        scope: &<SaveToFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.file_path.clone().into();

        let (hash, _size) = previous_context.unsigned_transaction.get_hash_and_size();
        let tx_bytes = borsh::to_vec(&previous_context.unsigned_transaction)
            .expect("Transaction serialization should not fail");
        let tx_base64 = near_primitives::serialize::to_base64(&tx_bytes);
        let data_unsigned_transaction = serde_json::json!({
            "Transaction hash to sign": hex::encode(hash.as_bytes()).to_string(),
            "Unsigned transaction (serialized as base64)": tx_base64,
        });

        std::fs::File::create(&file_path)
            .wrap_err_with(|| format!("Failed to create file: {:?}", &file_path))?
            .write(&serde_json::to_vec_pretty(&data_unsigned_transaction)?)
            .wrap_err_with(|| format!("Failed to write to file: {:?}", &file_path))?;

        if let crate::Verbosity::Quiet = previous_context.global_context.verbosity {
            return Ok(Self);
        }
        eprintln!(
            "\nThe file {:?} was created successfully. It has a unsigned transaction (serialized as base64).\nThis base64-encoded transaction can be signed and sent later. There is a helper command on near CLI that can do that:\n$ {} transaction sign-transaction",
            &file_path,
            crate::common::get_near_exec_path()
        );
        Ok(Self)
    }
}

impl SaveToFile {
    fn input_file_path(
        _context: &super::SignLaterContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the file path where to save the unsigned transaction:")
                .with_starting_input("unsigned-transaction-info.json")
                .prompt()?,
        ))
    }
}
