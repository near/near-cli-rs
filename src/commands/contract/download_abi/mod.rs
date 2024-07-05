use std::io::Write;

use color_eyre::eyre::Context;
use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Enter the name of the file to save the contract ABI:
    save_to_file: DownloadContractAbi,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl Contract {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = DownloadContractContext)]
pub struct DownloadContractAbi {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract ABI:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct DownloadContractContext(crate::network_view_at_block::ArgsForViewContext);

impl DownloadContractContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<DownloadContractAbi as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id = previous_context.account_id.clone();
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_abi(&account_id, &file_path, network_config, block_reference)
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![previous_context.account_id],
        }))
    }
}

impl From<DownloadContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: DownloadContractContext) -> Self {
        item.0
    }
}

impl DownloadContractAbi {
    fn input_file_path(
        context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the file path where the contract ABI should be saved to:")
                .with_starting_input(&format!(
                    "{}.abi.json",
                    context.account_id.as_str().replace('.', "_")
                ))
                .prompt()?,
        ))
    }
}

#[tracing::instrument(name = "Download the ABI for the contract ...", skip_all)]
fn download_contract_abi(
    account_id: &near_primitives::types::AccountId,
    file_path: &std::path::PathBuf,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
) -> crate::CliResult {
    let abi_root =
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(super::inspect::get_contract_abi(
                &network_config.json_rpc_client(),
                block_reference,
                account_id,
            ))?;
    std::fs::File::create(file_path)
        .wrap_err_with(|| format!("Failed to create file: {:?}", file_path))?
        .write(&serde_json::to_vec_pretty(&abi_root)?)
        .wrap_err_with(|| format!("Failed to write to file: {:?}", file_path))?;
    eprintln!("\nThe file {:?} was downloaded successfully", file_path);
    Ok(())
}
