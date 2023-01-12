mod add_key;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    ///What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

impl NewAccount {
    fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        super::fund_myself_create_account::NewAccount::input_new_account_id(context)
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.access_key_mode
            .process(config, self.new_account_id.clone())
            .await
    }
}
