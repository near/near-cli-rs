use near_primitives::types::{BlockId, BlockReference, Finality};
use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ArgsForViewContext)]
#[interactive_clap(output_context = NetworkViewAtBlockArgsContext)]
pub struct NetworkViewAtBlockArgs {
    /// What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    next: ViewAtBlock,
}

#[derive(Clone)]
pub struct NetworkViewAtBlockArgsContext {
    network_config: crate::config::NetworkConfig,
    on_after_getting_block_reference_callback: OnAfterGettingBlockReferenceCallback,
}

impl NetworkViewAtBlockArgsContext {
    pub fn from_previous_context(
        previous_context: ArgsForViewContext,
        scope: &<NetworkViewAtBlockArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let networks = previous_context.config.networks.clone();
        let network_config = networks
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        Ok(Self {
            network_config,
            on_after_getting_block_reference_callback: previous_context
                .on_after_getting_block_reference_callback,
        })
    }
}

impl NetworkViewAtBlockArgs {
    fn input_network_name(
        context: &ArgsForViewContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&(context.config.clone(),))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkViewAtBlockArgsContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(skip_default_from_cli)]
/// Сhoose block for view
pub enum ViewAtBlock {
    #[strum_discriminants(strum(
        message = "now               - View properties in the final block"
    ))]
    /// View properties in the final block
    Now,
    #[strum_discriminants(strum(
        message = "at-block-height   - View properties in a height-selected block"
    ))]
    /// View properties in a height-selected block
    AtBlockHeight(AtBlockHeight),
    #[strum_discriminants(strum(
        message = "at-block-hash     - View properties in a hash-selected block"
    ))]
    /// View properties in a hash-selected block
    AtBlockHash(BlockIdHash),
}

impl interactive_clap::FromCli for ViewAtBlock {
    type FromCliContext = NetworkViewAtBlockArgsContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        mut optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let network_config = &context.network_config.clone();

        if optional_clap_variant.is_none() {
            match Self::choose_variant(context.clone()) {
                interactive_clap::ResultFromCli::Ok(cli_args) => {
                    optional_clap_variant = Some(cli_args)
                }
                result => return result,
            }
        }

        match optional_clap_variant {
            Some(CliViewAtBlock::Now) => {
                let block_reference = Finality::Final.into();

                match (context.on_after_getting_block_reference_callback)(
                    &network_config,
                    &block_reference,
                ) {
                    Ok(_) => (),
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(CliViewAtBlock::Now), err)
                    }
                };
                interactive_clap::ResultFromCli::Ok(CliViewAtBlock::Now)
            }
            Some(CliViewAtBlock::AtBlockHeight(inner_cli_args)) => {
                let cli_inner_args = <AtBlockHeight as interactive_clap::FromCli>::from_cli(
                    Some(inner_cli_args.clone()),
                    context.clone().into(),
                );
                match cli_inner_args {
                    interactive_clap::ResultFromCli::Ok(cli_args) => {
                        let block_id_height = cli_args.block_id_height.expect("Unexpected error");
                        let block_reference =
                            BlockReference::BlockId(BlockId::Height(block_id_height));

                        match (context.on_after_getting_block_reference_callback)(
                            &network_config,
                            &block_reference,
                        ) {
                            Ok(_) => (),
                            Err(err) => {
                                return interactive_clap::ResultFromCli::Err(
                                    Some(CliViewAtBlock::AtBlockHeight(cli_args)),
                                    err,
                                )
                            }
                        };
                        interactive_clap::ResultFromCli::Ok(CliViewAtBlock::AtBlockHeight(cli_args))
                    }
                    interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
                    interactive_clap::ResultFromCli::Cancel(Some(cli_args)) => {
                        interactive_clap::ResultFromCli::Cancel(Some(
                            CliViewAtBlock::AtBlockHeight(cli_args),
                        ))
                    }
                    interactive_clap::ResultFromCli::Cancel(None) => {
                        interactive_clap::ResultFromCli::Cancel(None)
                    }
                    interactive_clap::ResultFromCli::Err(Some(cli_args), err) => {
                        interactive_clap::ResultFromCli::Err(
                            Some(CliViewAtBlock::AtBlockHeight(cli_args)),
                            err,
                        )
                    }
                    interactive_clap::ResultFromCli::Err(None, err) => {
                        interactive_clap::ResultFromCli::Err(None, err)
                    }
                }
            }
            Some(CliViewAtBlock::AtBlockHash(inner_cli_args)) => {
                let cli_inner_args = <BlockIdHash as interactive_clap::FromCli>::from_cli(
                    Some(inner_cli_args.clone()),
                    context.clone().into(),
                );
                match cli_inner_args {
                    interactive_clap::ResultFromCli::Ok(cli_args) => {
                        let block_id_hash =
                            cli_args.block_id_hash.clone().expect("Unexpected error");
                        let block_reference = BlockReference::BlockId(BlockId::Hash(
                            near_primitives::hash::CryptoHash::from_str(block_id_hash.as_str())
                                .unwrap(),
                        ));

                        match (context.on_after_getting_block_reference_callback)(
                            &network_config,
                            &block_reference,
                        ) {
                            Ok(_) => (),
                            Err(err) => {
                                return interactive_clap::ResultFromCli::Err(
                                    Some(CliViewAtBlock::AtBlockHash(cli_args)),
                                    err,
                                )
                            }
                        };
                        interactive_clap::ResultFromCli::Ok(CliViewAtBlock::AtBlockHash(cli_args))
                    }
                    interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
                    interactive_clap::ResultFromCli::Cancel(Some(cli_args)) => {
                        interactive_clap::ResultFromCli::Cancel(Some(CliViewAtBlock::AtBlockHash(
                            cli_args,
                        )))
                    }
                    interactive_clap::ResultFromCli::Cancel(None) => {
                        interactive_clap::ResultFromCli::Cancel(None)
                    }
                    interactive_clap::ResultFromCli::Err(Some(cli_args), err) => {
                        interactive_clap::ResultFromCli::Err(
                            Some(CliViewAtBlock::AtBlockHash(cli_args)),
                            err,
                        )
                    }
                    interactive_clap::ResultFromCli::Err(None, err) => {
                        interactive_clap::ResultFromCli::Err(None, err)
                    }
                }
            }
            None => unreachable!("Unexpected error"),
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkViewAtBlockArgsContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct AtBlockHeight {
    /// Type the block ID height
    block_id_height: near_primitives::types::BlockHeight,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkViewAtBlockArgsContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct BlockIdHash {
    /// Type the block ID hash
    block_id_hash: String,
}

pub type OnAfterGettingBlockReferenceCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig, &BlockReference) -> crate::CliResult>;

#[derive(Clone)]
pub struct ArgsForViewContext {
    pub config: crate::config::Config,
    pub on_after_getting_block_reference_callback: OnAfterGettingBlockReferenceCallback,
}
