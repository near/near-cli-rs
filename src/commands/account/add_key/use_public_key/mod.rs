#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
pub struct AddAccessKeyAction {
    ///Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct AddAccessKeyActionContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    public_key: crate::types::public_key::PublicKey,
}

impl AddAccessKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        scope: &<AddAccessKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            public_key: scope.public_key.clone(),
        }
    }
}

impl From<AddAccessKeyActionContext> for crate::commands::ActionContext {
    fn from(item: AddAccessKeyActionContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id.clone(),
            receiver_account_id: item.signer_account_id,
            actions: vec![near_primitives::transaction::Action::AddKey(
                near_primitives::transaction::AddKeyAction {
                    public_key: item.public_key.into(),
                    access_key: near_primitives::account::AccessKey {
                        nonce: 0,
                        permission: item.permission,
                    },
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(|prepolulated_unsinged_transaction, network_config| {
                Ok(())
            }),
            on_after_signing_callback: std::sync::Arc::new(|singed_transaction| {
                Ok(())
            }),
        }
    }
}

impl AddAccessKeyAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        let access_key = near_primitives::account::AccessKey {
            nonce: 0,
            permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: self.public_key.clone().into(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.network_config.get_network_config(config),
            ),
            None => Ok(()),
        }
    }
}
