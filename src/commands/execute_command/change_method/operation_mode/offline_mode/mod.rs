use dialoguer::Input;

/// аргументы, необходимые для offline mode
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOfflineArgs {
    #[clap(long)]
    nonce: Option<u64>,
    #[clap(long)]
    block_hash: Option<crate::common::BlockHashAsBase58>,
    #[clap(subcommand)]
    pub send_to: Option<super::super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct OfflineArgs {
    nonce: u64,
    block_hash: near_primitives::hash::CryptoHash,
    send_to: super::super::receiver::SendTo,
}

impl From<CliOfflineArgs> for OfflineArgs {
    fn from(item: CliOfflineArgs) -> Self {
        let nonce: u64 = match item.nonce {
            Some(cli_nonce) => cli_nonce,
            None => OfflineArgs::input_nonce(),
        };
        let block_hash = match item.block_hash {
            Some(cli_block_hash) => cli_block_hash.inner,
            None => OfflineArgs::input_block_hash(),
        };
        let send_to = match item.send_to {
            Some(cli_send_to) => super::super::receiver::SendTo::from(cli_send_to),
            None => super::super::receiver::SendTo::send_to(),
        };
        Self {
            nonce,
            block_hash,
            send_to,
        }
    }
}

impl OfflineArgs {
    fn input_nonce() -> u64 {
        Input::new()
            .with_prompt(
                "Enter transaction nonce (query the access key information with \
                `near-cli utils view-access-key frol4.testnet ed25519:...` incremented by 1)",
            )
            .interact_text()
            .unwrap()
    }

    fn input_block_hash() -> near_primitives::hash::CryptoHash {
        let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
            .with_prompt("Enter recent block hash")
            .interact_text()
            .unwrap();
        input_block_hash.inner
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let nonce = self.nonce.clone();
        let block_hash = self.block_hash.clone();
        let unsigned_transaction = near_primitives::transaction::Transaction {
            block_hash,
            nonce,
            ..prepopulated_unsigned_transaction
        };
        let selected_server_url = None;
        self.send_to
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
