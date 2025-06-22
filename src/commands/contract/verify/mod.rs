use color_eyre::{
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
};

use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use near_verify_rs::types::{
    contract_source_metadata::ContractSourceMetadata,
    source_id::{GitReference, SourceId, SourceKind},
    whitelist::{Whitelist, WhitelistEntry},
};

use crate::common::JsonRpcClientExt;
use crate::types::contract_properties::ContractProperties;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    use_contract_source_code_path: Option<crate::types::path_buf::PathBuf>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    save_contract_source_code_into: Option<crate::types::path_buf::PathBuf>,
    #[interactive_clap(long)]
    no_image_whitelist: bool,
    #[interactive_clap(subcommand)]
    source_contract_code: SourceContractCode,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    use_contract_source_code_path: Option<std::path::PathBuf>,
    save_contract_source_code_into: Option<std::path::PathBuf>,
    no_image_whitelist: bool,
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
            use_contract_source_code_path: scope
                .use_contract_source_code_path
                .as_ref()
                .map(std::path::PathBuf::from),
            save_contract_source_code_into: scope
                .save_contract_source_code_into
                .as_ref()
                .map(std::path::PathBuf::from),
            no_image_whitelist: scope.no_image_whitelist,
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
                let contract_code_from_contract_account_id = get_contract_code_from_contract_account_id(&account_id, network_config, block_reference)?;
                let contract_properties = get_contract_properties_from_repository(
                    &account_id,
                    network_config,
                    block_reference,
                    previous_context.use_contract_source_code_path.clone(),
                    previous_context.save_contract_source_code_into.clone(),
                    previous_context.no_image_whitelist
                )?;

                verify_contract(
                    previous_context.global_context.verbosity.clone(),
                    contract_code_from_contract_account_id,
                    contract_properties,
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

fn verify_contract(
    verbosity: crate::Verbosity,
    contract_code: Vec<u8>,
    contract_properties: ContractProperties,
) {
    if contract_properties.code == contract_code {
        if let crate::Verbosity::Quiet = verbosity {
            println!(
                "The code obtained from the contract account ID and the code calculated from the repository are the same.\n{}",
                contract_properties
            )
        } else {
            tracing::info!("{}\n{}",
                "The code obtained from the contract account ID and the code calculated from the repository are the same.".green(),
                crate::common::indent_payload(&contract_properties.to_string())
            )
        }
    } else if let crate::Verbosity::Quiet = verbosity {
        println!("The code obtained from the contract account ID and the code calculated from the repository do not match.")
    } else {
        tracing::info!("{}", "The code obtained from the contract account ID and the code calculated from the repository do not match.".red())
    }
}

#[tracing::instrument(
    name = "Getting the contract properties from the repository ...",
    skip_all
)]
fn get_contract_properties_from_repository(
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
    use_contract_source_code_path: Option<std::path::PathBuf>,
    save_contract_source_code_into: Option<std::path::PathBuf>,
    no_image_whitelist: bool,
) -> color_eyre::eyre::Result<ContractProperties> {
    let contract_source_metadata = tokio::runtime::Runtime::new().unwrap().block_on(
        super::inspect::get_contract_source_metadata(
            &network_config.json_rpc_client(),
            block_reference,
            account_id,
        ),
    )?;

    get_contract_properties_from_docker_build(
        contract_source_metadata,
        use_contract_source_code_path,
        save_contract_source_code_into,
        no_image_whitelist,
    )
}

#[tracing::instrument(name = "Getting contract properties from docker build ...", skip_all)]
fn get_contract_properties_from_docker_build(
    contract_source_metadata: ContractSourceMetadata,
    use_contract_source_code_path: Option<std::path::PathBuf>,
    save_contract_source_code_into: Option<std::path::PathBuf>,
    no_image_whitelist: bool,
) -> color_eyre::eyre::Result<ContractProperties> {
    let whitelist: Option<Whitelist> = if no_image_whitelist {
        None
    } else {
        Some(vec![WhitelistEntry {
            expected_docker_image: "sourcescan/cargo-near".to_string(),
        }])
    };
    contract_source_metadata.validate(whitelist)?;

    let build_info = contract_source_metadata.build_info.as_ref().wrap_err("`contract_source_metadata` does not have a `build_info` field. This field is an addition to version **1.2.0** of **NEP-330**.")?;
    let source_id = SourceId::from_url(&build_info.source_code_snapshot)?;

    let tempdir = tempfile::tempdir()?;

    let target_dir = if let Some(path_buf) = save_contract_source_code_into {
        path_buf
    } else if let Some(path_buf) = use_contract_source_code_path.clone() {
        path_buf
    } else {
        tempdir.path().to_path_buf()
    };

    if use_contract_source_code_path.is_none() {
        let SourceKind::Git(GitReference::Rev(rev)) = source_id.kind();
        checkout_remote_repo(source_id.url().as_str(), &target_dir, rev)?;
    }

    let target_dir = camino::Utf8PathBuf::from_path_buf(target_dir)
        .map_err(|err| color_eyre::eyre::eyre!("convert path buf {:?}", err))?;

    let contract_path_buf = tracing_indicatif::suspend_tracing_indicatif::<
        _,
        color_eyre::eyre::Result<camino::Utf8PathBuf>,
    >(|| {
        near_verify_rs::logic::nep330_build::run(
            contract_source_metadata.clone(),
            target_dir,
            vec![],
            false,
        )
    })?;
    let contract_code = std::fs::read(&contract_path_buf)
        .wrap_err_with(|| format!("Failed to open or read the file: {:?}.", &contract_path_buf,))?;
    let contract_code_hash = near_verify_rs::logic::compute_hash(contract_path_buf)?;

    let contract_properties = ContractProperties {
        code: contract_code,
        hash: contract_code_hash,
        version: contract_source_metadata.version,
        standards: contract_source_metadata.standards,
        link: contract_source_metadata.link,
        source: build_info.source_code_snapshot.clone(),
        build_environment: build_info.build_environment.clone(),
        build_command: build_info.build_command.clone(),
    };

    Ok(contract_properties)
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

#[tracing::instrument(name = "Getting the contract code from", skip_all)]
fn get_contract_code_from_contract_account_id(
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    tracing::Span::current().pb_set_message(&format!("{account_id} ..."));
    tracing::info!(target: "near_teach_me", "{}", format!("{account_id} ..."));
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
            "I am making HTTP call to NEAR JSON RPC to get the contract code (Wasm binary) deployed to `{}` account, learn more https://docs.near.org/api/rpc/contracts#view-contract-code",
            account_id
    );
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
