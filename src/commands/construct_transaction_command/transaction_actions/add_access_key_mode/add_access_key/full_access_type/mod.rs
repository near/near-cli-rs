use async_recursion::async_recursion;

/// данные для определения ключа с полным доступом
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliFullAccessType {
    #[clap(subcommand)]
    next_action: Option<super::super::super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct FullAccessType {
    pub next_action: Box<super::super::super::NextAction>,
}

impl interactive_clap::ToCli for FullAccessType {
    type CliVariant = CliFullAccessType;
}

impl CliFullAccessType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<FullAccessType> for CliFullAccessType {
    fn from(_full_access_type: FullAccessType) -> Self {
        Self {
            next_action: Some(super::super::super::CliSkipNextAction::Skip(
                super::super::super::CliSkipAction { sign_option: None },
            )),
        }
    }
}

impl FullAccessType {
    pub fn from_cli(
        optional_clap_variant: Option<CliFullAccessType>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let skip_next_action: super::super::super::NextAction = match optional_clap_variant
            .and_then(|clap_variant| clap_variant.next_action)
        {
            Some(cli_skip_action) => super::super::super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                context,
            )?,
            None => super::super::super::NextAction::choose_variant(context)?,
        };
        Ok(Self {
            next_action: Box::new(skip_next_action),
        })
    }
}

impl FullAccessType {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            super::super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
