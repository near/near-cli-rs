use async_recursion::async_recursion;
use dialoguer::Input;

/// удаление аккаунта
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAccountAction {
    #[clap(long)]
    beneficiary_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountAction {
    pub beneficiary_id: near_primitives::types::AccountId,
    pub next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for DeleteAccountAction {
    type CliVariant = CliDeleteAccountAction;
}

impl CliDeleteAccountAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(beneficiary_id) = &self.beneficiary_id {
            args.push_front(beneficiary_id.to_string());
            args.push_front("--beneficiary-id".to_owned())
        };
        args
    }
}

impl From<DeleteAccountAction> for CliDeleteAccountAction {
    fn from(delete_account_action: DeleteAccountAction) -> Self {
        Self {
            beneficiary_id: Some(delete_account_action.beneficiary_id),
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl DeleteAccountAction {
    pub fn from_cli(
        optional_clap_variant: Option<CliDeleteAccountAction>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_id: near_primitives::types::AccountId = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.beneficiary_id)
        {
            Some(cli_account_id) => cli_account_id,
            None => DeleteAccountAction::input_beneficiary_id(&context)?,
        };
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            beneficiary_id,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl DeleteAccountAction {
    pub fn input_beneficiary_id(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter the beneficiary ID to delete this account ID")
            .interact_text()
            .unwrap())
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let beneficiary_id: near_primitives::types::AccountId = self.beneficiary_id.clone();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction { beneficiary_id },
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
