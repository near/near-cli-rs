#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ContractFile {
    ///What is a file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    submit: Option<super::Submit>,
}

impl ContractFile {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let code = std::fs::read(&self.file_path.0.clone()).map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to open or read the file: {:?}.\nError: {:?}",
                &self.file_path.0.clone(),
                err
            ))
        })?;
