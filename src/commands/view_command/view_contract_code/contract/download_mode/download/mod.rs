use dialoguer::Input;
use std::io::Write;

// download contract file
#[derive(Debug, Default, clap::Clap)]
pub struct CliContractFile {
    file_path: Option<std::path::PathBuf>,
}

#[derive(Debug)]
pub struct ContractFile {
    pub file_path: Option<std::path::PathBuf>,
}

impl ContractFile {
    pub fn from(item: CliContractFile, contract_id: &str) -> Self {
        let file_path = match item.file_path {
            Some(cli_file_path) => Some(cli_file_path),
            None => ContractFile::input_file_path(contract_id),
        };
        ContractFile { file_path }
    }
}

impl ContractFile {
    fn input_file_path(contract_id: &str) -> Option<std::path::PathBuf> {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("Where to download the contract file?")
            .with_initial_text(format!("{}.wasm", contract_id))
            .interact_text()
            .unwrap();
        Some(input_file_path.into())
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        contract_id: String,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewCode {
                    account_id: contract_id,
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
        match &self.file_path {
            Some(file_path) => {
                std::fs::File::create(file_path)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                    })?
                    .write(&call_access_view.code)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                    })?;
                println!(
                    "\nThe file {:?} was downloaded successfully",
                    self.file_path
                );
            }
            None => {
                println!("\nHash of the contract: {}", &call_access_view.hash)
            }
        }
        Ok(())
    }
}
