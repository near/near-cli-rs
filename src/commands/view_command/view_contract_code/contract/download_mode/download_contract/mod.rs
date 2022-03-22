use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct ContractFile {
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    pub selected_block_id: super::super::super::block_id::BlockId,
}

impl ContractFile {
    fn input_file_path(
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::path_buf::PathBuf> {
        println!();
        let contract_account_id = context.signer_account_id.clone();
        let input_file_path: String = Input::new()
            .with_prompt("Where to download the contract file?")
            .with_initial_text(format!("{}.wasm", contract_account_id))
            .interact_text()?;
        let file_path = if cfg!(unix) {
            shellexpand::tilde(&input_file_path).as_ref().parse()?
        } else {
            input_file_path.parse()?
        };
        Ok(file_path)
    }

    pub async fn process(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(
                contract_id,
                network_connection_config,
                Some(self.file_path.into()),
            )
            .await
    }
}
