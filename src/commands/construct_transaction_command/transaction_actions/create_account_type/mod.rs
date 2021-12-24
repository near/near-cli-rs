use async_recursion::async_recursion;

/// создание аккаунта
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCreateAccountAction {
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct CreateAccountAction {
    pub next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for CreateAccountAction {
    type CliVariant = CliCreateAccountAction;
}

impl CliCreateAccountAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<CreateAccountAction> for CliCreateAccountAction {
    fn from(_create_account_action: CreateAccountAction) -> Self {
        Self {
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl CreateAccountAction {
    pub fn from_cli(
        optional_clap_variant: Option<<CreateAccountAction as interactive_clap::ToCli>::CliVariant>,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            next_action: Box::new(skip_next_action),
        })
    }
}

impl CreateAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
