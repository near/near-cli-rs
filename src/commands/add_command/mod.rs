use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod access_key;
mod contract_code;
mod implicit_account;
mod stake_proposal;
mod sub_account;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct AddAction {
    #[interactive_clap(subcommand)]
    pub action: Action,
}

impl AddAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
/// What do you want to add?
pub enum Action {
    #[strum_discriminants(strum(message = "Add a new access key for an account"))]
    ///Add a new access key for an account
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new contract code"))]
    ///Add a contract code
    ContractCode(self::contract_code::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add an implicit-account"))]
    ///Add implicit account
    ImplicitAccount(self::implicit_account::ImplicitAccount),
    #[strum_discriminants(strum(message = "Add a new stake proposal"))]
    ///Add a stake proposal
    StakeProposal(self::stake_proposal::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new sub-account"))]
    ///Add a new sub-account
    SubAccount(self::sub_account::operation_mode::OperationMode),
}

impl Action {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Action::AccessKey(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::ContractCode(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::ImplicitAccount(generate_keypair) => generate_keypair.process().await,
            Action::StakeProposal(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::SubAccount(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
