use dialoguer::Input;


#[derive(Debug, clap::Clap)]
pub enum CliTransfer {
    /// Enter an amount
    Amount(CliTransferNEARTokensAction),
}

#[derive(Debug)]
pub enum Transfer {
    Amount(TransferNEARTokensAction),
}

impl From<CliTransfer> for Transfer {
    fn from(item: CliTransfer) -> Self {
        match item {
            CliTransfer::Amount(cli_transfer_near_action) => {
                Self::Amount(cli_transfer_near_action.into())
            }
        }
    }
}

impl Transfer {
    pub fn choose_transfer_near() -> Self {
        Self::from(CliTransfer::Amount(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            Transfer::Amount(transfer_near_action) => {
                transfer_near_action
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}

/// создание перевода токенов
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransferNEARTokensAction {
    amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    sign_transactions: Option<super::transactions_signing::CliTransactionsSigning>,
}

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub sign_transactions: super::transactions_signing::TransactionsSigning,
}

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: crate::common::NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => TransferNEARTokensAction::input_amount(),
        };
        let sign_transactions = match item.sign_transactions {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => super::transactions_signing::TransactionsSigning::choose_sign_transactions(),
        };
        Self {
            amount,
            sign_transactions,
        }
    }
}

impl TransferNEARTokensAction {
    pub fn input_amount() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
    }
    
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let stake = match self.amount {
            crate::common::NearBalance {inner: num} => num,
        };
        self.sign_transactions.process(prepopulated_unsigned_transaction, selected_server_url, stake).await
    }
}
