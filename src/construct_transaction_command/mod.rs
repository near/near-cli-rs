use structopt::StructOpt;
use strum::{EnumMessage, EnumDiscriminants, EnumIter, IntoEnumIterator};
use dialoguer::{
    Select,
    theme::ColorfulTheme,
};

mod select_on_off_line_mode;
use select_on_off_line_mode::{CliOnOffLineMode, Mode, OnOffLineMode};

mod sender;
mod receiver;
mod transaction_actions;
mod sign_transaction;
use super::utils_command::sign_transaction_subcommand;

#[derive(Debug, StructOpt)]
pub enum CliCommand {
    ConstructTransaction(CliOnOffLineMode),
    Utils(CliUtilType),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ArgsCommand {
    #[strum_discriminants(strum(message="Construct a new transaction"))]
    ConstructTransaction(OnOffLineMode),
    #[strum_discriminants(strum(message="Helpers"))]
    Utils(UtilType),
}

#[derive(Debug)]
pub struct UtilType {
    util: UtilList
}

#[derive(Debug, StructOpt)]
pub struct CliUtilType {
    #[structopt(subcommand)]
    util: Option<CliUtilList>
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
enum UtilList {
    #[strum_discriminants(strum(message="Sign a transaction"))]
    SignTransactionCommand(sign_transaction_subcommand::SignTransaction)
}

#[derive(Debug, StructOpt)]
enum CliUtilList {
    SignTransactionCommand(sign_transaction_subcommand::CliSignTransaction)
}

impl From<CliUtilType> for UtilType {
    fn from(item: CliUtilType) -> Self {
        let util: UtilList = match item.util {
            Some(cli_util) => UtilList::from(cli_util),
            None => UtilList::choose_util()
        };
        UtilType {util}
    }
}

impl UtilList {
    fn process(self) {
        println!("--- UtilList: process {:?} ", &self);
        match self {
            UtilList::SignTransactionCommand(sign_transaction) => sign_transaction.process(),
            _ => unreachable!("Error")
        }
    }
    fn choose_util() -> Self {
        println!();
        let variants = UtilListDiscriminants::iter().collect::<Vec<_>>();
        let utils = variants.iter().map(|p| p.get_message().unwrap().to_owned()).collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&utils)
            .default(0)
            .interact()
            .unwrap();
        match variants[selection] {
            UtilListDiscriminants::SignTransactionCommand => {
                let signer_secret_key = sign_transaction_subcommand::SignTransaction::input_signer_secret_key();
                let unsigned_transaction = sign_transaction_subcommand::SignTransaction::input_unsigned_transaction();
                Self::SignTransactionCommand(sign_transaction_subcommand::SignTransaction {
                    signer_secret_key,
                    unsigned_transaction
                })
            },
        }
    }
}

impl From<CliUtilList> for UtilList {
    fn from(item: CliUtilList) -> Self {
        match item {
            CliUtilList::SignTransactionCommand(cli_sign_transaction) => {
                let sign_transaction = sign_transaction_subcommand::SignTransaction::from(cli_sign_transaction);
                UtilList::SignTransactionCommand(sign_transaction)
            } 
        }
    }
}

impl From<CliCommand> for ArgsCommand {
    fn from(item: CliCommand) -> Self {
        match item {
            CliCommand::ConstructTransaction(cli_onoffline_mode) => {
                let onoffline_mode = OnOffLineMode::from(cli_onoffline_mode);
                ArgsCommand::ConstructTransaction(onoffline_mode)
            }
            CliCommand::Utils(cli_util_type) => {
                let util_type = UtilType::from(cli_util_type);
                ArgsCommand::Utils(util_type)
            },
        }
    }
}

impl ArgsCommand {
    pub fn choose_command() -> Self {
        println!();
        let variants = ArgsCommandDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants.iter().map(|p| p.get_message().unwrap().to_owned()).collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&commands)
            .default(0)
            .interact()
            .unwrap();
        match variants[selection] {
            ArgsCommandDiscriminants::ConstructTransaction => {
                Self::ConstructTransaction(OnOffLineMode{mode: Mode::choose_mode()})
            },
            ArgsCommandDiscriminants::Utils => {
                Self::Utils(UtilType{util: UtilList::choose_util()})
            },
        }
    }
}

impl UtilType {
    pub fn process(self) {
        println!("=== Util Type: {:?}", &self);
        self.util.process()
    }
}
