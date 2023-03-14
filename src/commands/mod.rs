use near_primitives::transaction;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod account;
mod config;
// mod contract;
mod tokens;
// mod transaction;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
// #[interactive_clap(skip_default_from_cli)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "account     - Manage accounts"))]
    /// View account summary, create subaccount, delete account, list keys, add key, delete key, import account
    Account(self::account::AccountCommands),
    #[strum_discriminants(strum(
        message = "tokens      - Manage token assets such as NEAR, FT, NFT"
    ))]
    /// Use this for token actions: send or view balances of NEAR, FT, or NFT
    Tokens(self::tokens::TokensCommands),
    // #[strum_discriminants(strum(
    //     message = "contract    - Manage smart-contracts: deploy code, call functions"
    // ))]
    // /// Use this for contract actions: call function, deploy, download wasm, inspect storage
    // Contract(self::contract::ContractCommands),
    // #[strum_discriminants(strum(message = "transaction - Operate transactions"))]
    // /// Use this to construct transactions or view a transaction status.
    // Transaction(self::transaction::TransactionCommands),
    #[strum_discriminants(strum(
        message = "config      - Manage connections in a configuration file (config.toml)"
    ))]
    /// Use this to manage connections in a configuration file (config.toml).
    Config(self::config::ConfigCommands),
}

// impl interactive_clap::FromCli for TopLevelCommand {
//     type FromCliContext = crate::GlobalContext;
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
//         match optional_clap_variant {
//             Some(clap_variant) => {
//                 println!("----------------- clap_variant: {:?}", &clap_variant);

//                 match <self::tokens::TokensCommands as interactive_clap::FromCli>::from_cli(
//                     None,
//                     context.clone(),
//                 ) {
//                     interactive_clap::ResultFromCli::Ok(cli_args) => {
//                         interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Tokens(cli_args))
//                     }
//                     interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                         interactive_clap::ResultFromCli::Cancel(Some(CliTopLevelCommand::Tokens(
//                             optional_cli_args.unwrap_or_default(),
//                         )))
//                     }
//                     interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
//                     interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                         interactive_clap::ResultFromCli::Err(
//                             Some(CliTopLevelCommand::Tokens(
//                                 optional_cli_args.unwrap_or_default(),
//                             )),
//                             err,
//                         )
//                     }
//                 }

//                 // interactive_clap::ResultFromCli::Ok(clap_variant)
//             }
//             None => choose_variant(context.into()),
//         }
//     }
// }

// impl interactive_clap::FromCli for TopLevelCommand {
//     type FromCliContext = crate::GlobalContext;
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
//         match optional_clap_variant {
//             Some(CliTopLevelCommand::Account(inner_cli_args)) => {
//                 let optional_inner_args =
//                     <self::account::AccountCommands as interactive_clap::FromCli>::from_cli(
//                         Some(inner_cli_args),
//                         context.clone().into(),
//                     );
//                     if let interactive_clap::ResultFromCli::Ok(cli_args) = optional_inner_args {
//                         interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Account(cli_args))
//                     } else {
//                         Self::choose_variant(context.clone())
//                     }
//                 }
//             Some(CliTopLevelCommand::Tokens(inner_cli_args)) => {
//                 let cli_inner_args =
//                     <self::tokens::TokensCommands as interactive_clap::FromCli>::from_cli(
//                         Some(inner_cli_args),
//                         context.clone().into(),
//                     );
//                 // if let interactive_clap::ResultFromCli::Ok(cli_args) = optional_inner_args {
//                 //     interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Tokens(cli_args))
//                 // } else {
//                 //     Self::choose_variant(context.clone())
//                 // }

//                 match cli_inner_args {
//                     interactive_clap::ResultFromCli::Ok(cli_args) => {
//                         interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Tokens(cli_args))
//                     }
//                     interactive_clap::ResultFromCli::Back => {
//                         interactive_clap::ResultFromCli::Back
//                     }
//                     interactive_clap::ResultFromCli::Cancel(Some(cli_args)) => {
//                         interactive_clap::ResultFromCli::Cancel(Some(CliTopLevelCommand::Tokens(cli_args)))
//                     }
//                     interactive_clap::ResultFromCli::Cancel(None) => {
//                         interactive_clap::ResultFromCli::Cancel(None)
//                     }
//                     interactive_clap::ResultFromCli::Err(Some(cli_args), err) => {
//                         interactive_clap::ResultFromCli::Err(Some(CliTopLevelCommand::Tokens(cli_args)), err)
//                     }
//                     interactive_clap::ResultFromCli::Err(None, err) => {
//                         interactive_clap::ResultFromCli::Err(None, err)
//                     }
//                 }

//             }
//             Some(CliTopLevelCommand::Config(inner_cli_args)) => {
//                 let optional_inner_args =
//                     <self::config::ConfigCommands as interactive_clap::FromCli>::from_cli(
//                         Some(inner_cli_args),
//                         context.clone().into(),
//                     );
//                     if let interactive_clap::ResultFromCli::Ok(cli_args) = optional_inner_args {
//                         interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Config(cli_args))
//                     } else {
//                         Self::choose_variant(context.clone())
//                     }
//                 }
//             None => Self::choose_variant(context.into()),
//         }
//     }
// }

// pub fn choose_variant(
//     context: crate::GlobalContext,
// ) -> interactive_clap::ResultFromCli<
//     <TopLevelCommand as interactive_clap::ToCli>::CliVariant,
//     <TopLevelCommand as interactive_clap::FromCli>::FromCliError,
// > {
//     use inquire::Select;
//     use interactive_clap::SelectVariantOrBack;
//     use strum::{EnumMessage, IntoEnumIterator};
//     loop {
//         println!("====================");
//         let selected_variant = Select :: new (" What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)" , TopLevelCommandDiscriminants :: iter () . map (SelectVariantOrBack :: Variant) . collect ()) . prompt () ;
//         match selected_variant {
//             Ok(SelectVariantOrBack::Variant(variant)) => match variant {
//                 TopLevelCommandDiscriminants::Account => {
//                     match <self::account::AccountCommands as interactive_clap::FromCli>::from_cli(
//                         None,
//                         context.clone(),
//                     ) {
//                         interactive_clap::ResultFromCli::Ok(cli_args) => {
//                             return interactive_clap::ResultFromCli::Ok(
//                                 CliTopLevelCommand::Account(cli_args),
//                             )
//                         }
//                         interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                             return interactive_clap::ResultFromCli::Cancel(Some(
//                                 CliTopLevelCommand::Account(optional_cli_args.unwrap_or_default()),
//                             ));
//                         }
//                         interactive_clap::ResultFromCli::Back => continue,
//                         interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                             return interactive_clap::ResultFromCli::Err(
//                                 Some(CliTopLevelCommand::Account(
//                                     optional_cli_args.unwrap_or_default(),
//                                 )),
//                                 err,
//                             );
//                         }
//                     }
//                 }
//                 TopLevelCommandDiscriminants::Tokens => {
//                     match <self::tokens::TokensCommands as interactive_clap::FromCli>::from_cli(
//                         None,
//                         context.clone(),
//                     ) {
//                         interactive_clap::ResultFromCli::Ok(cli_args) => {
//                             return interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Tokens(
//                                 cli_args,
//                             ))
//                         }
//                         interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                             return interactive_clap::ResultFromCli::Cancel(Some(
//                                 CliTopLevelCommand::Tokens(optional_cli_args.unwrap_or_default()),
//                             ));
//                         }
//                         interactive_clap::ResultFromCli::Back => continue,
//                         interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                             return interactive_clap::ResultFromCli::Err(
//                                 Some(CliTopLevelCommand::Tokens(
//                                     optional_cli_args.unwrap_or_default(),
//                                 )),
//                                 err,
//                             );
//                         }
//                     }
//                 }
//                 TopLevelCommandDiscriminants::Config => {
//                     match <self::config::ConfigCommands as interactive_clap::FromCli>::from_cli(
//                         None,
//                         context.clone(),
//                     ) {
//                         interactive_clap::ResultFromCli::Ok(cli_args) => {
//                             return interactive_clap::ResultFromCli::Ok(CliTopLevelCommand::Config(
//                                 cli_args,
//                             ))
//                         }
//                         interactive_clap::ResultFromCli::Cancel(optional_cli_args) => {
//                             return interactive_clap::ResultFromCli::Cancel(Some(
//                                 CliTopLevelCommand::Config(optional_cli_args.unwrap_or_default()),
//                             ));
//                         }
//                         interactive_clap::ResultFromCli::Back => continue,
//                         interactive_clap::ResultFromCli::Err(optional_cli_args, err) => {
//                             return interactive_clap::ResultFromCli::Err(
//                                 Some(CliTopLevelCommand::Config(
//                                     optional_cli_args.unwrap_or_default(),
//                                 )),
//                                 err,
//                             );
//                         }
//                     }
//                 }
//             },
//             Ok(SelectVariantOrBack::Back) => return interactive_clap::ResultFromCli::Back,
//             Err(
//                 inquire::error::InquireError::OperationCanceled
//                 | inquire::error::InquireError::OperationInterrupted,
//             ) => return interactive_clap::ResultFromCli::Cancel(None),
//             Err(err) => return interactive_clap::ResultFromCli::Err(None, err.into()),
//         }
//     }
// }

impl TopLevelCommand {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::Tokens(tokens_commands) => tokens_commands.process(config).await,
            Self::Account(account_commands) => account_commands.process(config).await,
            // Self::Contract(contract_commands) => contract_commands.process(config).await,
            // Self::Transaction(transaction_commands) => transaction_commands.process(config).await,
            Self::Config(config_commands) => config_commands.process(config).await,
        }
    }
}

pub type OnBeforeSigningCallback = std::sync::Arc<
    dyn Fn(
        &mut near_primitives::transaction::Transaction,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;
pub type OnAfterGettingNetworkCallback = std::sync::Arc<
    dyn Fn(
        &mut near_primitives::transaction::Transaction,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;

#[derive(Clone)]
pub struct ActionContext {
    pub config: crate::config::Config,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId, // maybe it should be removed and transferred to callback
    pub actions: Vec<near_primitives::transaction::Action>,
    pub on_after_getting_network_callback: OnAfterGettingNetworkCallback,
    pub on_before_signing_callback: OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

#[derive(Clone)]
pub struct TransactionContext {
    pub config: crate::config::Config,
    pub network_config: crate::config::NetworkConfig,
    pub transaction: near_primitives::transaction::Transaction,
    pub on_before_signing_callback: OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}
