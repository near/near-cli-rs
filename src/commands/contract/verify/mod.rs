use std::io::Write;

use color_eyre::{
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
};

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use near_verify_rs::types::{
    contract_source_metadata::ContractSourceMetadata,
    source_id::{GitReference, SourceId, SourceKind},
};

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    use_contract_source_code_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    save_contract_source_code_into: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    #[interactive_clap(subcommand)]
    source_contract_code: SourceContractCode,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    use_contract_source_code_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    save_contract_source_code_into: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if scope.use_contract_source_code_path.is_some()
            & scope.save_contract_source_code_into.is_some()
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "You are using the `--use-contract-source-code-path` and `--save-contract-source-code-into` options at the same time. This is not allowed."
            ));
        }
        Ok(Self {
            global_context: previous_context,
            use_contract_source_code_path: scope.use_contract_source_code_path.clone(),
            save_contract_source_code_into: scope.save_contract_source_code_into.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ContractContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose the code source for the contract:
pub enum SourceContractCode {
    #[strum_discriminants(strum(
        message = "deployed-at    - Verify the contract by contract account ID"
    ))]
    /// Verify the contract by contract account ID
    DeployedAt(ContractAccountId),
    #[strum_discriminants(strum(
        message = "wasm-file      - Verify the contract using a wasm file"
    ))]
    /// Verify the contract using a wasm file
    WasmFile(ContractFile),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractAccountIdContext)]
pub struct ContractAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ContractAccountId {
    pub fn input_contract_account_id(
        context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Clone)]
pub struct ContractAccountIdContext(crate::network_view_at_block::ArgsForViewContext);

impl ContractAccountIdContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_primitives::types::AccountId = scope.contract_account_id.clone().into();

            move |network_config, block_reference| {
                let contract_source_code = if let Some(path) = &previous_context.use_contract_source_code_path {
                    path.read_bytes()?
                } else {
                    let docker_build_code = get_contract_code_from_repository(&account_id, network_config, block_reference)?;
                    if let Some(path) = &previous_context.save_contract_source_code_into {
                        std::fs::File::create(path)
                            .wrap_err_with(|| format!("Failed to create file: {:?}", &path))?
                            .write(&docker_build_code)
                            .wrap_err_with(|| format!("Failed to write to file: {:?}", &path))?;
                        tracing::info!("The file '{}' was downloaded successfully", path);
                    }
                    docker_build_code
                };

                verify_contract(
                    contract_source_code,
                    get_contract_code_from_contract_account_id(&account_id, network_config, block_reference)?
                );

                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![scope.contract_account_id.clone().into()],
        }))
    }
}

impl From<ContractAccountIdContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ContractAccountIdContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is a file location of the contract?
    pub file_path: crate::types::utf8_path_buf::Utf8PathBuf,
}

pub struct ContractFileContext;

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let wasm_code = scope.file_path.read_bytes()?;

        let contract_source_code = if let Some(path) =
            previous_context.use_contract_source_code_path
        {
            path.read_bytes()?
        } else {
            let contract_source_metadata = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(fetch_contract_source_metadata_from_wasm(&wasm_code))?;

            let (_tempdir, docker_build_out_wasm) =
                get_docker_build_out_wasm_from_contract_source_metadata(contract_source_metadata)?;

            let docker_build_code = std::fs::read(&docker_build_out_wasm).wrap_err_with(|| {
                format!(
                    "Failed to open or read the file: {:?}.",
                    &docker_build_out_wasm,
                )
            })?;
            if let Some(path) = previous_context.save_contract_source_code_into {
                std::fs::File::create(&path)
                    .wrap_err_with(|| format!("Failed to create file: {:?}", &path))?
                    .write(&docker_build_code)
                    .wrap_err_with(|| format!("Failed to write to file: {:?}", &path))?;
                tracing::info!("The file '{}' was downloaded successfully", path);
            }
            docker_build_code
        };

        verify_contract(contract_source_code, wasm_code);

        Ok(Self)
    }
}

fn verify_contract(contract_code_from_repository: Vec<u8>, contract_code: Vec<u8>) {
    // XXX fix message
    if contract_code_from_repository == contract_code {
        tracing::info!("\n{}", crate::common::indent_payload(&
            format!(
            "The hash code obtained from the contract account ID and the hash code calculated from the repository are the same.\nhash: ",
            // contract_code_hash
        )))
    } else {
        tracing::info!("\n{}", crate::common::indent_payload(&
            format!(
            "The hash code obtained from the contract account ID: \nThe hash code calculated from the repository:        ",
            // contract_code_hash,
            // contract_code_hash_from_repository.to_base58_string()
        )).red())
    };
}

#[tracing::instrument(
    name = "Getting the contract code hash from the repository ...",
    skip_all
)]
fn get_contract_code_from_repository(
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    let contract_source_metadata = tokio::runtime::Runtime::new().unwrap().block_on(
        super::inspect::get_contract_source_metadata(
            &network_config.json_rpc_client(),
            block_reference,
            account_id,
        ),
    )?;
    let (_tempdir, docker_build_out_wasm) =
        get_docker_build_out_wasm_from_contract_source_metadata(contract_source_metadata)?;

    std::fs::read(&docker_build_out_wasm).wrap_err_with(|| {
        format!(
            "Failed to open or read the file: {:?}.",
            &docker_build_out_wasm,
        )
    })
}

#[tracing::instrument(
    name = "Getting the docker build out wasm from the contract source metadata ...",
    skip_all
)]
fn get_docker_build_out_wasm_from_contract_source_metadata(
    contract_source_metadata: ContractSourceMetadata,
) -> color_eyre::eyre::Result<(tempfile::TempDir, camino::Utf8PathBuf)> {
    let build_info = contract_source_metadata.build_info.as_ref().wrap_err("`contract_source_metadata` does not have a `build_info` field. This field is an addition to version **1.2.0** of **NEP-330**.")?;
    let source_id = SourceId::from_url(&build_info.source_code_snapshot)?;

    let tempdir = tempfile::tempdir()?;
    let target_dir = tempdir.path().to_path_buf();

    let SourceKind::Git(GitReference::Rev(rev)) = source_id.kind();

    checkout_remote_repo(source_id.url().as_str(), &target_dir, rev)?;

    let target_dir = camino::Utf8PathBuf::from_path_buf(target_dir)
        .map_err(|err| color_eyre::eyre::eyre!("convert path buf {:?}", err))?;

    contract_source_metadata.validate(None)?;
    let utf8_path_buf = tracing_indicatif::suspend_tracing_indicatif::<
        _,
        color_eyre::eyre::Result<camino::Utf8PathBuf>,
    >(|| {
        near_verify_rs::logic::nep330_build::run(
            contract_source_metadata,
            target_dir,
            vec![],
            false,
        )
    })?;
    Ok((tempdir, utf8_path_buf))
}

fn checkout_remote_repo(
    repo_url: &str,
    target_path: &std::path::Path,
    rev_str: &str,
) -> crate::CliResult {
    let repo = git2::Repository::clone_recurse(repo_url, target_path)?;
    let oid = git2::Oid::from_str(rev_str)?;
    let _commit = repo.find_commit(oid)?;
    let object = repo.revparse_single(rev_str)?;
    repo.checkout_tree(&object, None)?;

    Ok(())
}

#[tracing::instrument(
    name = "Getting the contract code hash from the contract account ID ...",
    skip_all
)]
fn get_contract_code_from_contract_account_id(
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    let view_code_response = network_config
        .json_rpc_client()
        .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: block_reference.clone(),
            request: near_primitives::views::QueryRequest::ViewCode {
                account_id: account_id.clone(),
            },
        })
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query ViewCode for <{}> on network <{}>",
                &account_id, network_config.network_name
            )
        })?;

    let contract_code_view =
        if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
            view_code_response.kind
        {
            result
        } else {
            return Err(color_eyre::Report::msg("Error call result".to_string()));
        };
    Ok(contract_code_view.code)
}

async fn fetch_contract_source_metadata_from_wasm(
    wasm: &[u8],
) -> color_eyre::eyre::Result<ContractSourceMetadata> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(wasm).await?;
    let outcome = contract.view("contract_source_metadata").await?;
    Ok(serde_json::from_slice::<ContractSourceMetadata>(
        outcome.result.as_slice(),
    )?)
}
