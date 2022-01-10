use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct AddAccessKeyAction {
    pub public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    ///Enter an amount
    pub deposit: super::super::super::deposit::TransferNEARTokensAction,
}

impl AddAccessKeyAction {
    fn input_public_key(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
        Ok(Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: self.public_key.clone().into(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.deposit
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
