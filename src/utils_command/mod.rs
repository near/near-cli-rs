use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

// mod generate_keypair_subcommand;
mod sign_transaction_subcommand;

#[derive(Debug)]
pub struct UtilType {
    pub util: UtilList,
}

#[derive(Debug, StructOpt)]
pub struct CliUtilType {
    #[structopt(subcommand)]
    util: Option<CliUtilList>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum UtilList {
    #[strum_discriminants(strum(message = "Sign a transaction"))]
    SignTransactionCommand(sign_transaction_subcommand::SignTransaction),
}

#[derive(Debug, StructOpt)]
enum CliUtilList {
    SignTransactionCommand(sign_transaction_subcommand::CliSignTransaction),
}

impl From<CliUtilType> for UtilType {
    fn from(item: CliUtilType) -> Self {
        let util: UtilList = match item.util {
            Some(cli_util) => UtilList::from(cli_util),
            None => UtilList::choose_util(),
        };
        UtilType { util }
    }
}

impl UtilList {
    pub fn process(self) {
        match self {
            UtilList::SignTransactionCommand(sign_transaction) => sign_transaction.process(),
            _ => unreachable!("Error"),
        }
    }
    pub fn choose_util() -> Self {
        println!();
        let variants = UtilListDiscriminants::iter().collect::<Vec<_>>();
        let utils = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&utils)
            .default(0)
            .interact()
            .unwrap();
        match variants[selection] {
            UtilListDiscriminants::SignTransactionCommand => {
                let signer_secret_key =
                    sign_transaction_subcommand::SignTransaction::input_signer_secret_key();
                let unsigned_transaction =
                    sign_transaction_subcommand::SignTransaction::input_unsigned_transaction();
                Self::SignTransactionCommand(sign_transaction_subcommand::SignTransaction {
                    signer_secret_key,
                    unsigned_transaction,
                })
            }
        }
    }
}

impl From<CliUtilList> for UtilList {
    fn from(item: CliUtilList) -> Self {
        match item {
            CliUtilList::SignTransactionCommand(cli_sign_transaction) => {
                let sign_transaction =
                    sign_transaction_subcommand::SignTransaction::from(cli_sign_transaction);
                UtilList::SignTransactionCommand(sign_transaction)
            }
        }
    }
}

impl UtilType {
    pub fn process(self) {
        self.util.process()
    }
}
