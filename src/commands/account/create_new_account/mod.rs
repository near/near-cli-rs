use dialoguer::Input;
use serde_json::json;
use std::str::FromStr;

mod add_key;
mod deposit;

#[derive(Debug, Clone, Default)]
pub struct AccountProperties {
    pub new_account_id: Option<crate::types::account_id::AccountId>,
    pub public_key: crate::types::public_key::PublicKey,
    pub deposit: crate::common::NearBalance,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct NewAccount {
    ///What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Enter deposit for a function call
    attached_deposit: self::deposit::Deposit,
}

impl NewAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let account_properties = AccountProperties {
            new_account_id: Some(self.new_account_id.clone()),
            ..Default::default()
        };
        self.attached_deposit
            .process(config, account_properties)
            .await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignerAccountId {
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: AccountProperties,
    ) -> crate::CliResult {
        let mut new_account_id = account_properties
            .new_account_id
            .expect("Impossible to get account_id!");
        let network_config = self.network_config.get_network_config(config.clone());

        let account_id = loop {
            if (crate::common::get_account_state(
                network_config.clone(),
                new_account_id.clone().into(),
                near_primitives::types::Finality::Final.into(),
            )
            .await?)
                .is_some()
            {
                println!("Account <{}> already exists", new_account_id);
            } else {
                break new_account_id;
            }
            new_account_id = Input::new()
                .with_prompt("Enter a new account name")
                .interact_text()?;
        };

        let args = json!({
            "new_account_id": account_id.to_string(),
            "new_public_key": account_properties.public_key.to_string()
        })
        .to_string()
        .into_bytes();
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: network_config
                .clone()
                .linkdrop_account_id
                .expect("Impossible to get linkdrop_account_id!"),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "create_account".to_string(),
                    args,
                    gas: crate::common::NearGas::from_str("30 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: account_properties.deposit.to_yoctonear(),
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
            Some(transaction_info) => {
                if !matches!(
                    transaction_info.status,
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_)
                ) {
                    return crate::common::print_transaction_status(
                        transaction_info,
                        self.network_config.get_network_config(config),
                    );
                }
                println!("New account <{}> created successfully.", account_id);
                println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                    id=transaction_info.transaction_outcome.id,
                    path=self.network_config.get_network_config(config).explorer_transaction_url
                );
                Ok(())
            }
            None => Ok(()),
        }
    }
}