use async_recursion::async_recursion;
use dialoguer::Input;


/// данные для создания ставки
#[derive(Debug, Default, clap::Clap)]
pub struct CliStakeNEARTokensAction {
    stake: Option<crate::common::NearBalance>,
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug)]
pub struct StakeNEARTokensAction {
    pub stake: crate::common::NearBalance,
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<super::NextAction>,
}

impl From<CliStakeNEARTokensAction> for StakeNEARTokensAction {
    fn from(item: CliStakeNEARTokensAction) -> Self {
        let stake: crate::common::NearBalance = match item.stake {
            Some(cli_stake) => cli_stake,
            None => StakeNEARTokensAction::input_stake(),
        };
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => StakeNEARTokensAction::input_public_key(),
        };
        let skip_next_action: super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::NextAction::from(cli_skip_action),
            None => super::NextAction::input_next_action(),
        };
        Self {
            stake,
            public_key,
            next_action: Box::new(skip_next_action),
        }
    }
}

impl StakeNEARTokensAction {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this stake")
            .interact_text()
            .unwrap()
    }
    
    fn input_stake() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake?")
            .interact_text()
            .unwrap()
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let stake = match self.stake {
            crate::common::NearBalance {inner: num} => num,
        };
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake,
                public_key: self.public_key.clone()
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
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}
