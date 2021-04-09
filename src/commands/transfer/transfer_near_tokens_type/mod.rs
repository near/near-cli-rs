use dialoguer::Input;


/// создание перевода токенов
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransferNEARTokensAction {
    #[clap(long)]
    amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    mode: Option<super::operation_mode::CliMode>,
}

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub mode: super::operation_mode::Mode,
}

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: crate::common::NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => TransferNEARTokensAction::input_amount(),
        };
        let mode = match item.mode {
            Some(cli_mode) => super::operation_mode::Mode::from(cli_mode),
            None => super::operation_mode::Mode::choose_mode()
        };
        Self {
            amount,
            mode,
        }
    }
}

impl TransferNEARTokensAction {
    fn input_amount() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
    }
    
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let amount = match self.amount {
            crate::common::NearBalance {inner: num} => num,
        };
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction { deposit: amount },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.mode.process(unsigned_transaction).await
    }
}
