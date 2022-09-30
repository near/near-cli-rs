mod initialize_mode;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Contract {
    ///What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Specify a path to wasm file
    use_file: ContractFile,
}

impl Contract {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.use_file
            .process(config, self.account_id.clone().into())
            .await
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ContractFile {
    ///What is a file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    initialize: self::initialize_mode::InitializeMode,
}

impl ContractFile {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let code = std::fs::read(&self.file_path.0.clone()).map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to open or read the file: {:?}.\nError: {:?}",
                &self.file_path.0.clone(),
                err
            ))
        })?;
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: account_id.clone(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: account_id,
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::DeployContract(
                near_primitives::transaction::DeployContractAction { code },
            )],
        };
        self.initialize
            .process(config, prepopulated_unsigned_transaction)
            .await
    }
}
