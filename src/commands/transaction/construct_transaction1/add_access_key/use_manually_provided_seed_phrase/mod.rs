use async_recursion::async_recursion;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AddAccessWithSeedPhraseAction {
    ///Enter the seed_phrase for this sub-account
    master_seed_phrase: String,
    #[interactive_clap(subcommand)]
    next_action: super::super::BoxNextAction,
}

impl AddAccessWithSeedPhraseAction {
    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        let seed_phrase_hd_path_default = slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            seed_phrase_hd_path_default,
            &self.master_seed_phrase,
        )?;
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action.clone().inner {
            super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
