use inquire::Text;
use std::io::Write;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractAccountContext)]
pub struct ContractAccount {
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select a folder to download the contract
    to_folder: DownloadContract,
}

#[derive(Debug, Clone)]
pub struct ContractAccountContext {
    config: crate::config::Config,
    account_id: near_primitives::types::AccountId,
}

impl ContractAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ContractAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            account_id: scope.account_id.clone().into(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractAccountContext)]
#[interactive_clap(output_context = DownloadContractContext)]
pub struct DownloadContract {
    #[interactive_clap(skip_default_input_arg)]
    ///Where to download the contract file?
    folder_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct DownloadContractContext(crate::network_view_at_block::ArgsForViewContext);

impl DownloadContractContext {
    pub fn from_previous_context(
        previous_context: ContractAccountContext,
        scope: &<DownloadContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = previous_context.account_id;
        let folder_path: std::path::PathBuf = scope.folder_path.clone().into();

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            move |network_config, block_reference| {

                let query_view_method_response = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(
                 network_config
                .json_rpc_client()
                .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: block_reference.clone(),
                    request: near_primitives::views::QueryRequest::ViewCode {
                        account_id: account_id.clone(),
                    },
                })
            )
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to fetch query for view contract: {:?}",
                        err
                    ))
                })?;
            let call_access_view =
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                    query_view_method_response.kind
                {
                    result
                } else {
                    return Err(color_eyre::Report::msg("Error call result".to_string()));
                };
            let mut file_path = folder_path.clone();
            std::fs::create_dir_all(&file_path)?;
            let file_name: std::path::PathBuf =
                format!("contract_{}.wasm", account_id.as_str().replace('.', "_")).into();
            file_path.push(file_name);
            std::fs::File::create(&file_path)
                .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
                .write(&call_access_view.code)
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                })?;
            println!("\nThe file {:?} was downloaded successfully", &file_path);

            Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<DownloadContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: DownloadContractContext) -> Self {
        item.0
    }
}

impl DownloadContract {
    fn input_folder_path(
        _context: &ContractAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let mut folder_path = std::path::PathBuf::from(&home_dir);
        folder_path.push("Downloads");
        println!();
        let input_folder_path = Text::new("Where to download the contract file?")
            .with_initial_value(&format!("{}", folder_path.to_string_lossy()))
            .prompt()?;
        let folder_path = shellexpand::tilde(&input_folder_path).as_ref().parse()?;
        Ok(Some(folder_path))
    }
}
