use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

type CliResult = color_eyre::eyre::Result<()>;

mod common;
mod utils_command;
use utils_command::{CliUtils, Utils};
mod construct_transaction_command;
mod consts;
use construct_transaction_command::operation_mode::{CliOperationMode, OperationMode};

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
        let cli_subcommand = match item.subcommand {
            Some(cli_subcommand) => cli_subcommand,
            None => ArgsCommand::choose_command(),
        };
        Self { subcommand: ArgsCommand::from(cli_subcommand) }
    }
}

impl Args {
    async fn process(self) -> CliResult {
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
                mode.process(unsigned_transaction).await
            }
            ArgsCommand::Utils(util_type) => util_type.process(),
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum CliCommand {
    ConstructTransaction(CliOperationMode),
    Utils(CliUtils),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ArgsCommand {
    #[strum_discriminants(strum(message = "Construct a new transaction"))]
    ConstructTransaction(OperationMode),
    #[strum_discriminants(strum(message = "Helpers"))]
    Utils(Utils),
}

impl From<CliCommand> for ArgsCommand {
    fn from(item: CliCommand) -> Self {
        match item {
            CliCommand::ConstructTransaction(cli_operation_mode) => {
                let operation_mode = OperationMode::from(cli_operation_mode);
                ArgsCommand::ConstructTransaction(operation_mode)
            }
            CliCommand::Utils(cli_util) => {
                let util = Utils::from(cli_util);
                ArgsCommand::Utils(util)
            }
        }
    }
}

impl ArgsCommand {
    pub fn choose_command() -> CliCommand {
        println!();
        let variants = ArgsCommandDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&commands)
            .default(0)
            .interact()
            .unwrap();
        match variants[selection] {
            ArgsCommandDiscriminants::ConstructTransaction => {
                CliCommand::ConstructTransaction(Default::default())
            }
            ArgsCommandDiscriminants::Utils => {
                CliCommand::Utils(Default::default())
            },
        }
    }
}

fn main() -> CliResult {
    let cli = CliArgs::from_args();
    let args = Args::from(cli);

    color_eyre::install()?;

    actix::System::new().block_on(args.process())
}
