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
    config: crate::config::Config,
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
            config: previous_context.config,
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

    // pub fn get_network_config(
    //     &self,
    //     config: crate::config::Config,
    // ) -> crate::config::NetworkConfig {
    //     let network_config = config.networks;
    //     network_config
    //         .get(self.network_name.as_str())
    //         .expect("Impossible to get network name!")
    //         .clone()
    // }

    // pub fn get_block_ref(&self) -> BlockReference {
    //     match self.next.clone() {
    //         ViewAtBlock::Now => Finality::Final.into(),
    //         ViewAtBlock::AtBlockHash(at_block_hash) => BlockReference::BlockId(BlockId::Hash(
    //             near_primitives::hash::CryptoHash::from_str(at_block_hash.block_id_hash.as_str())
    //                 .unwrap(),
    //         )),
    //         ViewAtBlock::AtBlockHeight(at_block_height) => {
    //             BlockReference::BlockId(BlockId::Height(at_block_height.block_id_height))
    //         }
    //     }
    // }
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
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let network_config = &context.network_config.clone();
        match optional_clap_variant {
            Some(CliViewAtBlock::Now) => interactive_clap::ResultFromCli::Ok(CliViewAtBlock::Now),
            Some(CliViewAtBlock::AtBlockHeight(inner_cli_args)) => {
                let cli_inner_args = <AtBlockHeight as interactive_clap::FromCli>::from_cli(
                    Some(inner_cli_args),
                    context.clone().into(),
                );
                match cli_inner_args {
                    interactive_clap::ResultFromCli::Ok(cli_args) => {
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
                    Some(inner_cli_args),
                    context.clone().into(),
                );
                match cli_inner_args {
                    interactive_clap::ResultFromCli::Ok(cli_args) => {
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
            None => {
                let variant = Self::choose_variant(context.clone().into());
                if let interactive_clap::ResultFromCli::Ok(cli_view_at_block) = &variant {
                    match cli_view_at_block {
                        CliViewAtBlock::Now => {
                            let block_reference = Finality::Final.into();

                            match (context.on_after_getting_block_reference_callback)(
                                &network_config,
                                &block_reference,
                            ) {
                                Ok(_) => (),
                                Err(err) => {
                                    return interactive_clap::ResultFromCli::Err(
                                        Some(CliViewAtBlock::Now),
                                        err,
                                    )
                                }
                            };
                        }
                        CliViewAtBlock::AtBlockHeight(cli_at_block_height) => {
                            let block_id_height = cli_at_block_height
                                .block_id_height
                                .expect("Unexpected error");
                            let block_reference =
                                BlockReference::BlockId(BlockId::Height(block_id_height));

                            match (context.on_after_getting_block_reference_callback)(
                                &network_config,
                                &block_reference,
                            ) {
                                Ok(_) => (),
                                Err(err) => {
                                    return interactive_clap::ResultFromCli::Err(
                                        Some(CliViewAtBlock::AtBlockHeight(
                                            cli_at_block_height.clone(),
                                        )),
                                        err,
                                    )
                                }
                            };
                        }
                        CliViewAtBlock::AtBlockHash(cli_block_id_hash) => {
                            let block_id_hash = cli_block_id_hash
                                .block_id_hash
                                .clone()
                                .expect("Unexpected error");
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
                                        Some(CliViewAtBlock::AtBlockHash(
                                            cli_block_id_hash.clone(),
                                        )),
                                        err,
                                    )
                                }
                            };
                        }
                    }
                }
                variant
            }
        }
    }
}

// fn choose_variant(
//     context: NetworkViewAtBlockArgsContext,
// ) -> interactive_clap::ResultFromCli<
//     <ViewAtBlock as interactive_clap::ToCli>::CliVariant,
//     <ViewAtBlock as interactive_clap::FromCli>::FromCliError,
// > {
//     use inquire::Select;
//     use interactive_clap::SelectVariantOrBack;
//     use strum::{EnumMessage, IntoEnumIterator};
//     loop {
//         let selected_variant = Select::new(
//             " Сhoose block for view",
//             ViewAtBlockDiscriminants::iter()
//                 .map(SelectVariantOrBack::Variant)
//                 .chain([SelectVariantOrBack::Back])
//                 .collect(),
//         )
//         .prompt();
//         match selected_variant {
//             Ok(SelectVariantOrBack::Variant(variant)) => match variant {
//                 ViewAtBlockDiscriminants::Now => {
//                     return interactive_clap::ResultFromCli::Ok(CliViewAtBlock::Now);
//                 }
//                 ViewAtBlockDiscriminants::AtBlockHeight => {
//                     match <AtBlockHeight as interactive_clap::FromCli>::from_cli(
//                         None,
//                         context.clone(),
//                     ) {
//                         interactive_clap::ResultFromCli::Ok(cli_args) => {
//                             return interactive_clap::ResultFromCli::Ok(
//                                 CliViewAtBlock::AtBlockHeight(cli_args),
//                             );
//                         }
//                         interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                             return interactive_clap::ResultFromCli::Cancel(Some(
//                                 CliViewAtBlock::AtBlockHeight(
//                                     optional_cli_args.unwrap_or_default(),
//                                 ),
//                             ));
//                         }
//                         interactive_clap::ResultFromCli::Back => continue,
//                         interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                             return interactive_clap::ResultFromCli::Err(
//                                 Some(CliViewAtBlock::AtBlockHeight(
//                                     optional_cli_args.unwrap_or_default(),
//                                 )),
//                                 err,
//                             );
//                         }
//                     }
//                 }
//                 ViewAtBlockDiscriminants::AtBlockHash => {
//                     match <BlockIdHash as interactive_clap::FromCli>::from_cli(
//                         None,
//                         context.clone(),
//                     ) {
//                         interactive_clap::ResultFromCli::Ok(cli_args) => {
//                             return interactive_clap::ResultFromCli::Ok(
//                                 CliViewAtBlock::AtBlockHash(cli_args),
//                             );
//                         }
//                         interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                             return interactive_clap::ResultFromCli::Cancel(Some(
//                                 CliViewAtBlock::AtBlockHash(optional_cli_args.unwrap_or_default()),
//                             ));
//                         }
//                         interactive_clap::ResultFromCli::Back => continue,
//                         interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                             return interactive_clap::ResultFromCli::Err(
//                                 Some(CliViewAtBlock::AtBlockHash(
//                                     optional_cli_args.unwrap_or_default(),
//                                 )),
//                                 err,
//                             );
//                         }
//                     }
//                 }
//             },
//             Ok(SelectVariantOrBack::Back) => {
//                 return interactive_clap::ResultFromCli::Back;
//             }
//             Err(
//                 inquire::error::InquireError::OperationCanceled
//                 | inquire::error::InquireError::OperationInterrupted,
//             ) => return interactive_clap::ResultFromCli::Cancel(None),
//             Err(err) => {
//                 return interactive_clap::ResultFromCli::Err(None, err.into());
//             }
//         }
//     }
// }

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkViewAtBlockArgsContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct AtBlockHeight {
    /// Type the block ID height
    block_id_height: near_primitives::types::BlockHeight,
}

// impl interactive_clap::FromCli for AtBlockHeight {
//     type FromCliContext = NetworkViewAtBlockArgsContext;
//     type FromCliError = color_eyre::eyre::Error;
//     fn from_cli(
//         optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
//         context: Self::FromCliContext,
//     ) -> interactive_clap::ResultFromCli<
//         <Self as interactive_clap::ToCli>::CliVariant,
//         Self::FromCliError,
//     >
//     where
//         Self: Sized + interactive_clap::ToCli,
//     {
//         let mut clap_variant = optional_clap_variant.unwrap_or_default();
//         if clap_variant.block_id_height.is_none() {
//             clap_variant.block_id_height = match Self::input_block_id_height(&context) {
//                 Ok(Some(block_id_height)) => Some(block_id_height),
//                 Ok(None) => {
//                     return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
//                 }
//                 Err(err) => {
//                     return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
//                 }
//             };
//         }
//         let block_id_height = clap_variant
//             .block_id_height
//             .clone()
//             .expect("Unexpected error");
//         let block_reference = BlockReference::BlockId(BlockId::Height(block_id_height));

//         (context.on_after_getting_block_reference_callback)(
//             &context.network_config,
//             &block_reference,
//         );

//         interactive_clap::ResultFromCli::Ok(clap_variant)
//     }
// }

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkViewAtBlockArgsContext)]
// #[interactive_clap(skip_default_from_cli)]
pub struct BlockIdHash {
    /// Type the block ID hash
    block_id_hash: String,
}

// impl interactive_clap::FromCli for BlockIdHash {
//     type FromCliContext = NetworkViewAtBlockArgsContext;
//     type FromCliError = color_eyre::eyre::Error;
//     fn from_cli(
//         optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
//         context: Self::FromCliContext,
//     ) -> interactive_clap::ResultFromCli<
//         <Self as interactive_clap::ToCli>::CliVariant,
//         Self::FromCliError,
//     >
//     where
//         Self: Sized + interactive_clap::ToCli,
//     {
//         let mut clap_variant = optional_clap_variant.unwrap_or_default();
//         if clap_variant.block_id_hash.is_none() {
//             clap_variant.block_id_hash = match Self::input_block_id_hash(&context) {
//                 Ok(Some(block_id_hash)) => Some(block_id_hash),
//                 Ok(None) => {
//                     return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
//                 }
//                 Err(err) => {
//                     return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
//                 }
//             };
//         }
//         let block_id_hash = clap_variant
//             .block_id_hash
//             .clone()
//             .expect("Unexpected error");
//         let block_reference = BlockReference::BlockId(BlockId::Hash(
//             near_primitives::hash::CryptoHash::from_str(block_id_hash.as_str()).unwrap(),
//         ));

//         (context.on_after_getting_block_reference_callback)(
//             &context.network_config,
//             &block_reference,
//         );

//         interactive_clap::ResultFromCli::Ok(clap_variant)
//     }
// }

pub type OnAfterGettingBlockReferenceCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig, &BlockReference) -> crate::CliResult>;

// #[derive(Clone)]
// pub struct BlockReferenceContext {
//     config: crate::config::Config,
//     on_after_getting_block_reference_callback: OnAfterGettingBlockReferenceCallback,
// }

#[derive(Clone)]
pub struct ArgsForViewContext {
    pub config: crate::config::Config,
    pub on_after_getting_block_reference_callback: OnAfterGettingBlockReferenceCallback,
}
