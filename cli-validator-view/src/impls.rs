use clap::Clap;
// use std::path::PathBuf;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use near_primitives::hash::CryptoHash;
use near_primitives::types::{AccountId, BlockHeight};

use near_cli_derive;
use near_cli_visual::{prompt_variant, PromptInput};
#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]

////////////////////////// CliBlockIdHeight ///////////////////////////////
pub struct CliBlockIdHeight {
    block_id_height: Option<BlockHeight>,
}

////////////////////////// CliBlockIdHash //////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
pub struct CliBlockIdHash {
    block_id_hash: Option<CryptoHash>,
}

/////////////////////// CliBlockId ////////////////////////

#[derive(Debug, Clone, EnumDiscriminants, Clap, near_cli_derive::Interactive)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CliBlockId {
    /// Specify a block ID final to view this account
    #[strum_discriminants(strum(message = "At final block"))]
    AtFinalBlock,
    /// Specify a block ID height to view this account
    #[strum_discriminants(strum(message = "At block height"))]
    AtBlockHeight(CliBlockIdHeight),
    /// Specify a block ID hash to view this account
    #[strum_discriminants(strum(message = "At block hash"))]
    AtBlockHash(CliBlockIdHash),
}

impl PromptInput for CliBlockId {
    fn prompt_input() -> Self {
        match prompt_variant::<CliBlockIdDiscriminants>("Choose your action") {
            CliBlockIdDiscriminants::AtFinalBlock => CliBlockId::AtFinalBlock,
            CliBlockIdDiscriminants::AtBlockHeight => CliBlockId::AtBlockHeight(Default::default()),
            CliBlockIdDiscriminants::AtBlockHash => CliBlockId::AtBlockHash(Default::default()),
        }
    }
}

///////////////////////////// CliContractHash /////////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliContractHash {
    #[clap(subcommand)]
    selected_block_id: Option<CliBlockId>,
}

////////////////////////////////////// CliDownloadMode //////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CliDownloadMode {
    /// Download a contract file
    #[strum_discriminants(strum(message = "Download a contract file"))]
    Download(CliContractFile),
    /// View a contract hash
    #[strum_discriminants(strum(message = "View a contract hash"))]
    Hash(CliContractHash),
}

impl PromptInput for CliDownloadMode {
    fn prompt_input() -> Self {
        match prompt_variant("Choose your action") {
            CliDownloadModeDiscriminants::Download => CliDownloadMode::Download(Default::default()),
            CliDownloadModeDiscriminants::Hash => CliDownloadMode::Hash(Default::default()),
        }
    }
}

///////////////////////////// CliContractFile ////////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliContractFile {
    // file_path: Option<PathBuf>,
    #[clap(subcommand)]
    selected_block_id: Option<CliBlockId>,
}

//////////////////////////// CliContract ///////////////////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliContract {
    pub contract_id: Option<AccountId>,
    // TODO: if we can add new promts just by adding new fields to the struct, why we are not using it more widely?
    // pub test: Option<AccountId>,
    #[clap(subcommand)]
    download_mode: Option<CliDownloadMode>,
}

//////////////////////////// Proposals ///////////////////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliProposals {
    // TODO: this structure is redundant.
// But it can have a worker/process function if all the data will be available.
}

//////////////////////////// Validators ///////////////////////////////////////

#[derive(Debug, Default, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliValidators {
    #[clap(subcommand)]
    epoch: Option<CliEpochCommand>,
}

#[derive(Debug, Clone, Clap, EnumDiscriminants, near_cli_derive::Interactive)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CliEpochCommand {
    #[strum_discriminants(strum(message = "View latest proposals"))]
    Latest,
    /// View validators by EpochId
    // EpochId(self::view_command::CliViewQueryRequest), //TODO
    /// View validators by BlockId
    #[strum_discriminants(strum(message = "View by Block Id"))]
    BlockId(CliBlockIdWrapper),
}

impl PromptInput for CliEpochCommand {
    fn prompt_input() -> Self {
        match prompt_variant("Choose the Epoch") {
            CliEpochCommandDiscriminants::Latest => CliEpochCommand::Latest,
            CliEpochCommandDiscriminants::BlockId => CliEpochCommand::BlockId(Default::default()),
        }
    }
}

#[derive(clap::Clap, Default, Debug, Clone, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    // setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliBlockIdWrapper {
    #[clap(subcommand)]
    cli_block_id: Option<CliBlockId>,
}

//////////////////////////////////////// CliSendTo ///////////////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
pub enum CliSendTo<T>
where
    T: Default,
{
    /// Specify a contract
    SendTo(T),
}

impl<T> PromptInput for CliSendTo<T>
where
    T: Default,
{
    fn prompt_input() -> Self {
        Self::SendTo(Default::default())
    }
}

/////////////////////////////////////////////////// CliServer ////////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliServer<T> {
    #[clap(subcommand)]
    pub send_to: Option<T>,
}

// Needed to ignore <T: Default> trait bound requirement
impl<T> Default for CliServer<T> {
    fn default() -> Self {
        Self {
            send_to: Default::default(),
        }
    }
}

////////////////////////// CliCustomServer //////////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    version,
    author,
    about,
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliCustomServer<T> {
    // #[clap(long)]
    // pub url: Option<AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send_to: Option<T>,
}

// Needed to ignore <T: Default> trait bound requirement
impl<T> Default for CliCustomServer<T> {
    fn default() -> Self {
        Self {
            send_to: Default::default(),
        }
    }
}

/////////////////////////////////// CliSelectServer //////////////////////////////////

#[derive(Debug, Clone, Clap, EnumDiscriminants, near_cli_derive::Interactive)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CliSelectServer<T>
where
    T: clap::Args + Clone + Default,
{
    /// предоставление данных для сервера https://rpc.testnet.near.org
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(CliServer<CliSendTo<T>>),
    /// предоставление данных для сервера https://rpc.mainnet.near.org
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(CliServer<CliSendTo<T>>),
    /// предоставление данных для сервера https://rpc.betanet.near.org
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(CliServer<CliSendTo<T>>),
    /// предоставление данных для сервера, указанного вручную
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(CliCustomServer<CliSendTo<T>>),
}

impl<T> PromptInput for CliSelectServer<T>
where
    T: clap::Args + Clone + Default,
{
    fn prompt_input() -> Self {
        match prompt_variant("") {
            CliSelectServerDiscriminants::Testnet => Self::Testnet(Default::default()),
            CliSelectServerDiscriminants::Mainnet => Self::Mainnet(Default::default()),
            CliSelectServerDiscriminants::Betanet => Self::Betanet(Default::default()),
            CliSelectServerDiscriminants::Custom => Self::Custom(Default::default()),
        }
    }
}

////////////////////////////////////// CliNetworkArgs //////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]

pub struct CliNetworkArgs<T> {
    #[clap(subcommand)]
    selected_server: Option<T>,
}

impl<T> Default for CliNetworkArgs<T> {
    fn default() -> Self {
        Self {
            selected_server: Default::default(),
        }
    }
}

////////////////////////////////////// CliMode //////////////////////////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
pub enum CliMode<T> {
    /// Execute a change method with online mode
    Network(T),
}

impl<T> PromptInput for CliMode<T>
where
    T: Default,
{
    fn prompt_input() -> Self {
        Self::Network(T::default())
    }
}

/////////////////////////////////// CliOperationMode ////////////////////////////////////////////

#[derive(Debug, Clone, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
pub struct CliOperationMode<T> {
    #[clap(subcommand)]
    mode: Option<T>,
}

impl<T> Default for CliOperationMode<T> {
    fn default() -> Self {
        Self {
            mode: Default::default(),
        }
    }
}

//////////////////////////// CliQueryRequest //////////////////////////////

#[derive(Debug, Clone, EnumDiscriminants, Clap, near_cli_derive::Interactive)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CliQueryRequest {
    // TODO: why are we describing this nested structure here? Why not in main?
    /// View properties for an account
    #[strum_discriminants(strum(message = "View properties for an account (TODO: delete)"))]
    AccountSummary(CliOperationMode<CliMode<CliNetworkArgs<CliSelectServer<CliContract>>>>),
    #[strum_discriminants(strum(message = "View proposals"))]
    Proposals(CliOperationMode<CliMode<CliNetworkArgs<CliSelectServer<CliProposals>>>>),
    #[strum_discriminants(strum(message = "View validators"))]
    Validators(CliOperationMode<CliMode<CliNetworkArgs<CliSelectServer<CliValidators>>>>),
}

impl PromptInput for CliQueryRequest {
    fn prompt_input() -> Self {
        match prompt_variant::<CliQueryRequestDiscriminants>("Choose your action") {
            CliQueryRequestDiscriminants::AccountSummary => {
                Self::AccountSummary(Default::default())
            }
            CliQueryRequestDiscriminants::Proposals => Self::Proposals(Default::default()),
            CliQueryRequestDiscriminants::Validators => Self::Validators(Default::default()),
        }
    }
}
