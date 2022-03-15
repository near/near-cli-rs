use async_recursion::async_recursion;
use dialoguer::Input;

/// deleting an access key from a user
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct DeleteAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for DeleteAccessKeyAction {
    type CliVariant = CliDeleteAccessKeyAction;
}

impl CliDeleteAccessKeyAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        }
        args
    }
}

impl From<DeleteAccessKeyAction> for CliDeleteAccessKeyAction {
    fn from(delete_access_key_action: DeleteAccessKeyAction) -> Self {
        Self {
            public_key: Some(delete_access_key_action.public_key),
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl DeleteAccessKeyAction {
    pub fn from_cli(
        optional_clap_variant: Option<
            <DeleteAccessKeyAction as interactive_clap::ToCli>::CliVariant,
        >,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key: near_crypto::PublicKey = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.public_key)
        {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyAction::input_public_key(&context)?,
        };
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            public_key,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl DeleteAccessKeyAction {
    pub fn input_public_key(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<near_crypto::PublicKey> {
        Ok(Input::new()
            .with_prompt("Enter the access key to remove it")
            .interact_text()?)
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key: self.public_key,
            },
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
