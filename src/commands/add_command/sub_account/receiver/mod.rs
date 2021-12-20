use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct SubAccount {
    pub sub_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Specify a full access key for the sub-account
    pub sub_account_full_access: super::full_access_key::SubAccountFullAccess,
}

impl SubAccount {
    fn input_sub_account_id(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        Ok(Input::new()
            .with_prompt("What is the sub-account ID?")
            .interact_text()
            .unwrap())
    }

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
            receiver_id: self.sub_account_id.clone().into(),
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.sub_account_full_access
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
