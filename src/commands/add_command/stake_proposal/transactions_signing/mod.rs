use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTransactionsSigning {
    /// Enter an public key
    TransactionsSigningPublicKey(CliTransactionsSigningAction),
}

#[derive(Debug, Clone)]
pub enum TransactionsSigning {
    TransactionsSigningPublicKey(TransactionsSigningAction),
}

impl CliTransactionsSigning {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::TransactionsSigningPublicKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("transactions-signing-public-key".to_owned());
                args
            }
        }
    }
}

impl From<TransactionsSigning> for CliTransactionsSigning {
    fn from(transactions_signing: TransactionsSigning) -> Self {
        match transactions_signing {
            TransactionsSigning::TransactionsSigningPublicKey(transactions_signing_action) => {
                Self::TransactionsSigningPublicKey(transactions_signing_action.into())
            }
        }
    }
}

impl TransactionsSigning {
    pub fn from(
        item: CliTransactionsSigning,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliTransactionsSigning::TransactionsSigningPublicKey(
                cli_transactions_signing_action,
            ) => Ok(Self::TransactionsSigningPublicKey(
                TransactionsSigningAction::from(
                    cli_transactions_signing_action,
                    connection_config,
                    sender_account_id,
                )?,
            )),
        }
    }
}

impl TransactionsSigning {
    pub fn choose_sign_transactions(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliTransactionsSigning::TransactionsSigningPublicKey(Default::default()),
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        match self {
            TransactionsSigning::TransactionsSigningPublicKey(transactions_sign_action) => {
                transactions_sign_action
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        stake,
                    )
                    .await
            }
        }
    }
}

/// данные о получателе транзакции
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliTransactionsSigningAction {
    transactions_signing_public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct TransactionsSigningAction {
    pub transactions_signing_public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliTransactionsSigningAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(transactions_signing_public_key) = &self.transactions_signing_public_key {
            args.push_front(transactions_signing_public_key.to_string());
        }
        args
    }
}

impl From<TransactionsSigningAction> for CliTransactionsSigningAction {
    fn from(transactions_signing_action: TransactionsSigningAction) -> Self {
        Self {
            transactions_signing_public_key: Some(
                transactions_signing_action.transactions_signing_public_key,
            ),
            sign_option: Some(transactions_signing_action.sign_option.into()),
        }
    }
}

impl TransactionsSigningAction {
    fn from(
        item: CliTransactionsSigningAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let transactions_signing_public_key: near_crypto::PublicKey =
            match item.transactions_signing_public_key {
                Some(cli_transactions_signing_public_key) => cli_transactions_signing_public_key,
                None => TransactionsSigningAction::input_public_key(),
            };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(cli_sign_transaction, connection_config, sender_account_id)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(connection_config, sender_account_id)?,
        };
        Ok(Self {
            transactions_signing_public_key,
            sign_option,
        })
    }
}

impl TransactionsSigningAction {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this server")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake,
                public_key: self.transactions_signing_public_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config.clone())
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                );
            }
            None => {}
        };
        Ok(())
    }
}
