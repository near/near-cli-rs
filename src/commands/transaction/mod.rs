use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod construct_transaction;
mod construct_transaction_1;
mod construct_transaction_2;
mod construct_transaction_3;
mod construct_transaction_4;
mod construct_transaction_finish;
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
/// Сhoose action for transaction
pub enum TransactionActions {
    #[strum_discriminants(strum(message = "view-status            - View a transaction status"))]
    /// Execute function (contract method)
    ViewStatus(self::view_status::TransactionInfo),
    #[strum_discriminants(strum(
        message = "construct-transaction  - Construct a new transaction"
    ))]
    /// Construct a new transaction
    ConstructTransaction(TransactionAccounts),
}

impl TransactionActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::ViewStatus(_) => Ok(()),
            Self::ConstructTransaction(_) => Ok(()),
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ConstructTransactionActionContext)]
pub struct TransactionAccounts {
    /// What is the sender account ID?
    sender_account_id: crate::types::account_id::AccountId,
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_actions: self::construct_transaction::NextAction,
}

#[derive(Clone)]
pub struct ConstructTransactionActionContext {
    pub config: crate::config::Config,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
}

impl ConstructTransactionActionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionAccounts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            signer_account_id: scope.sender_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            actions: vec![],
        })
    }
}
