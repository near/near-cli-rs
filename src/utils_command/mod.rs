use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

// mod generate_keypair_subcommand;
mod sign_transaction_subcommand;

#[derive(Debug)]
pub struct Utils {
    pub util: Util,
}

#[derive(Debug, StructOpt)]
pub struct CliUtils {
    #[structopt(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Sign a transaction"))]
    SignTransaction(self::sign_transaction_subcommand::SignTransaction),
}

#[derive(Debug, StructOpt)]
enum CliUtil {
    SignTransaction(self::sign_transaction_subcommand::CliSignTransaction),
}

impl From<CliUtils> for Utils {
    fn from(item: CliUtils) -> Self {
        let util: Util = match item.util {
            Some(cli_util) => Util::from(cli_util),
            None => Util::choose_util(),
        };
        Utils { util }
    }
}

impl Util {
    pub fn process(self) {
        match self {
            Util::SignTransaction(sign_transaction) => sign_transaction.process(),
            _ => unreachable!("Error"),
        }
    }
    pub fn choose_util() -> Self {
        println!();
        let variants = UtilDiscriminants::iter().collect::<Vec<_>>();
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
            UtilDiscriminants::SignTransaction => {
                let signer_secret_key =
                    sign_transaction_subcommand::SignTransaction::input_signer_secret_key();
                let unsigned_transaction =
                    sign_transaction_subcommand::SignTransaction::input_unsigned_transaction();
                Self::SignTransaction(sign_transaction_subcommand::SignTransaction {
                    signer_secret_key,
                    unsigned_transaction,
                })
            }
        }
    }
}

impl From<CliUtil> for Util {
    fn from(item: CliUtil) -> Self {
        match item {
            CliUtil::SignTransaction(cli_sign_transaction) => {
                let sign_transaction =
                    sign_transaction_subcommand::SignTransaction::from(cli_sign_transaction);
                Util::SignTransaction(sign_transaction)
            }
        }
    }
}

impl Utils {
    pub fn process(self) {
        self.util.process()
    }
}
