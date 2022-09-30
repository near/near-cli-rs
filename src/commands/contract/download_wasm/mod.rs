use dialoguer::Input;
use std::io::Write;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ContractAccount {
    ///What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select a folder to download the contract
    to_folder: DownloadContract,
}

impl ContractAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.to_folder
            .process(config, self.account_id.clone().into())
            .await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct DownloadContract {
    #[interactive_clap(skip_default_input_arg)]
    ///Where to download the contract file?
    folder_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    ///Select network
    network: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl DownloadContract {
    fn input_folder_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::path_buf::PathBuf> {
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let mut folder_path = std::path::PathBuf::from(&home_dir);
        folder_path.push("Downloads");
        println!();
        let input_folder_path: String = Input::new()
            .with_prompt("Where to download the contract file?")
            .with_initial_text(format!("{}", folder_path.to_string_lossy()))
            .interact_text()?;
        let folder_path = shellexpand::tilde(&input_folder_path).as_ref().parse()?;
        Ok(folder_path)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .network
            .get_network_config(config)
            .json_rpc_client()?
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: self.network.get_block_ref(),
                request: near_primitives::views::QueryRequest::ViewCode {
                    account_id: account_id.clone(),
                },
            })
            .await
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
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        let mut file_path = self.folder_path.0.clone();
        std::fs::create_dir_all(&file_path)?;
        let file_name: std::path::PathBuf =
            format!("contract_{}.wasm", account_id.as_str().replace(".", "_")).into();
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
}
