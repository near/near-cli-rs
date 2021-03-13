use structopt::StructOpt;
use strum::{EnumMessage, EnumDiscriminants, EnumIter, IntoEnumIterator};
use dialoguer::{
    Select,
    theme::ColorfulTheme,
};

mod common;
mod utils_command;
use utils_command::{
    UtilType,
    CliUtilType,
    UtilList
};
mod consts;
mod construct_transaction_command;
use construct_transaction_command::operation_mode::{
    CliOperationMode,
    OperationMode,
    Mode
};


#[derive(Debug)]
struct Args {
    subcommand: ArgsCommand,
}

#[derive(Debug, Default, StructOpt)]
struct CliArgs {
    #[structopt(subcommand)]
    subcommand: Option<CliCommand>,
}

impl From<CliArgs> for Args {
    fn from(item: CliArgs) -> Self {
        let subcommand = match item.subcommand {
            Some(cli_subcommand) => ArgsCommand::from(cli_subcommand),
            None => ArgsCommand::choose_command(),
        };
        Self {
            subcommand,
        }
    }
}

impl Args {
    async fn process(self) -> String {
        match self.subcommand {
            ArgsCommand::ConstructTransaction(mode) => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    signer_id: "".to_string(),
                    public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                    nonce: 0,
                    receiver_id: "".to_string(),
                    block_hash: Default::default(),
                    actions: vec![],
                };
                mode.process(unsigned_transaction).await;
            },
            ArgsCommand::Utils(util_type) => {
                util_type.process()
            },
        };
        "Ok".to_string()
    }
}

#[derive(Debug, StructOpt)]
pub enum CliCommand {
    ConstructTransaction(CliOperationMode),
    Utils(CliUtilType),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ArgsCommand {
    #[strum_discriminants(strum(message="Construct a new transaction"))]
    ConstructTransaction(OperationMode),
    #[strum_discriminants(strum(message="Helpers"))]
    Utils(UtilType),
}

impl From<CliCommand> for ArgsCommand {
    fn from(item: CliCommand) -> Self {
        match item {
            CliCommand::ConstructTransaction(cli_operation_mode) => {
                let operation_mode = OperationMode::from(cli_operation_mode);
                ArgsCommand::ConstructTransaction(operation_mode)
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
                Self::ConstructTransaction(OperationMode{mode: Mode::choose_mode()})
            },
            ArgsCommandDiscriminants::Utils => {
                Self::Utils(UtilType{util: UtilList::choose_util()})
            },
        }
    }
}

fn main() {
    let cli = CliArgs::from_args();
    let args = Args::from(cli);

    actix::System::builder()
    .build()
    .block_on(async move { args.process().await });
}
