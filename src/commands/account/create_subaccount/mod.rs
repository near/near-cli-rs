#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SubAccount {
    ///What is the sub-account ID?
    new_account_id: crate::types::account_id::AccountId,
    ///Enter the amount for the subaccount
    initial_balance: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    access_key_mode: super::add_key::AccessKeyMode,
}

impl SubAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self
                .new_account_id
                .clone()
                .get_owner_account_id_from_sub_account()
                .into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.new_account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![
                near_primitives::transaction::Action::CreateAccount(
                    near_primitives::transaction::CreateAccountAction {},
                ),
                near_primitives::transaction::Action::Transfer(
                    near_primitives::transaction::TransferAction {
                        deposit: self.initial_balance.to_yoctonear(),
                    },
                ),
            ],
        };
        self.access_key_mode
            .process(
                config,
                prepopulated_unsigned_transaction,
                near_primitives::account::AccessKeyPermission::FullAccess,
            )
            .await
    }
}
