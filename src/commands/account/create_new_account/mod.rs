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
        let args = json!({
            "new_account_id": account_properties.new_account_id.expect("Impossible to get contract_account_id!").to_string(),
            "new_public_key": account_properties.public_key.to_string()
        })
        .to_string()
        .into_bytes();
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self
                .network_config
                .get_network_config(config.clone())
                .network_name
                .parse()
                .unwrap(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "create_account".to_string(),
                    args,
                    gas: crate::common::NearGas::from_str("100 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: account_properties.deposit.to_yoctonear(),
                },
            )],
        };
        crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config,
        )
        .await
    }
}
