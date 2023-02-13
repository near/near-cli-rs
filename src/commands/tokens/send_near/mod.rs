use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
pub struct SendNearCommand {
    ///What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    ///Enter an amount to transfer
    amount_in_near: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SendNearCommandContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount_in_near: crate::common::NearBalance,
}

impl SendNearCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendNearCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.config,
            signer_account_id: previous_context.owner_account_id.into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            amount_in_near: scope.amount_in_near.clone(),
        }
    }
}

impl From<SendNearCommandContext> for crate::commands::ActionContext {
    fn from(item: SendNearCommandContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            receiver_account_id: item.receiver_account_id,
            actions: vec![near_primitives::transaction::Action::Transfer(
                near_primitives::transaction::TransferAction {
                    deposit: item.amount_in_near.to_yoctonear(),
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(|prepolulated_unsinged_transaction| {
                Ok(())
            }),
            on_after_signing_callback: std::sync::Arc::new(|singed_transaction| Ok(())),
            on_after_getting_network_connection_callback: std::sync::Arc::new(|network_config| network_config.clone()),
        }
    }
}

impl SendNearCommand {
    fn input_amount_in_near(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        let input_amount =
            CustomType::new("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)").prompt()?;
        Ok(input_amount)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: owner_account_id,
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.receiver_account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::Transfer(
                near_primitives::transaction::TransferAction {
                    deposit: self.amount_in_near.to_yoctonear(),
                },
            )],
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
