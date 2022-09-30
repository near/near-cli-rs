use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod construct_transaction;
mod view_status;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TransactionCommands {
    #[interactive_clap(subcommand)]
    transaction_actions: TransactionActions,
}

impl TransactionCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.transaction_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Сhoose action for transaction
pub enum TransactionActions {
    #[strum_discriminants(strum(message = "view-status            - View a transaction status"))]
    ///Execute function (contract method)
    ViewStatus(self::view_status::TransactionInfo),
    #[strum_discriminants(strum(
        message = "construct-transaction  - Construct a new transaction"
    ))]
    ///Construct a new transaction
    ConstructTransaction(TransactionAccounts),
}

impl TransactionActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::ViewStatus(transaction_info) => transaction_info.process(config).await,
            Self::ConstructTransaction(transaction_accounts) => {
                transaction_accounts.process(config).await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TransactionAccounts {
    ///What is the sender account ID?
    sender_account_id: crate::types::account_id::AccountId,
    ///What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_actions: self::construct_transaction::NextAction,
}

impl TransactionAccounts {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.receiver_account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![],
        };
        self.next_actions
            .process(config, prepopulated_unsigned_transaction)
            .await
    }
}
