use structopt::StructOpt;
use strum_macros::{
    EnumVariantNames,
};
// use strum::VariantNames;
use strum::{EnumMessage, EnumDiscriminants, EnumIter, IntoEnumIterator};
use dialoguer::{
    Select,
    theme::ColorfulTheme,
    console::Term
};

mod select_on_off_line_mode;
use select_on_off_line_mode::{CliOnOffLineMode, Mode, OnOffLineMode};
mod sender;
mod receiver;
mod transaction_actions;
mod sign_transaction;

#[derive(Debug, StructOpt)]
pub enum CliCommand {
    /// consrtuct a new transaction
    ConstructTransaction(CliOnOffLineMode),
    Utils,
}

// #[derive(Debug, EnumVariantNames)]
// pub enum ArgsCommand {
//     /// consrtuct a new transaction
//     ConstructTransaction(OnOffLineMode),
//     Utils,
// }

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ArgsCommand {
    #[strum_discriminants(strum(message="Construct a new transaction"))]
    ConstructTransaction(OnOffLineMode),
    #[strum_discriminants(strum(message="Helpers"))]
    Utils,
}


impl From<CliCommand> for ArgsCommand {
    fn from(item: CliCommand) -> Self {
        match item {
            CliCommand::ConstructTransaction(cli_onoffline_mode) => {
                let onoffline_mode = OnOffLineMode::from(cli_onoffline_mode);
                ArgsCommand::ConstructTransaction(onoffline_mode)
            }
            CliCommand::Utils => ArgsCommand::Utils,
        }
    }
}

impl ArgsCommand {
    pub fn choose_command() -> Self {
        println!();
        // let commands = ArgsCommand::VARIANTS;
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
                Self::Utils
            },
            _ => unreachable!("Error")
        }
    }
}

