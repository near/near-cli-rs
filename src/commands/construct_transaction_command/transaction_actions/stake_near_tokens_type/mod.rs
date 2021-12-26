use async_recursion::async_recursion;
use dialoguer::Input;

/// создание ставки
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliStakeNEARTokensAction {
    stake_amount: Option<crate::common::NearBalance>,
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct StakeNEARTokensAction {
    pub stake_amount: crate::common::NearBalance,
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for StakeNEARTokensAction {
    type CliVariant = CliStakeNEARTokensAction;
}

impl CliStakeNEARTokensAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        };
        if let Some(stake_amount) = &self.stake_amount {
            args.push_front(stake_amount.to_string());
        };
        args
    }
}

impl From<StakeNEARTokensAction> for CliStakeNEARTokensAction {
    fn from(stake_near_tokens_action: StakeNEARTokensAction) -> Self {
        Self {
            stake_amount: Some(stake_near_tokens_action.stake_amount.into()),
            public_key: Some(stake_near_tokens_action.public_key),
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl StakeNEARTokensAction {
    pub fn from_cli(
        optional_clap_variant: Option<
            <StakeNEARTokensAction as interactive_clap::ToCli>::CliVariant,
        >,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let stake_amount: crate::common::NearBalance = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.stake_amount)
        {
            Some(cli_amount) => cli_amount,
            None => StakeNEARTokensAction::input_stake_amount(&context)?,
        };
        let public_key: near_crypto::PublicKey = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.public_key)
        {
            Some(cli_public_key) => cli_public_key,
            None => StakeNEARTokensAction::input_public_key(&context)?,
        };
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            stake_amount,
            public_key,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl StakeNEARTokensAction {
    fn input_public_key(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<near_crypto::PublicKey> {
        Ok(Input::new()
            .with_prompt("Enter a public key for this stake")
            .interact_text()?)
    }

    fn input_stake_amount(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        Ok(Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            ?)
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake: self.stake_amount.to_yoctonear(),
                public_key: self.public_key.clone(),
            },
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
