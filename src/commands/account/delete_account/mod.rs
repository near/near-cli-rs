#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct DeleteAccount {
    ///What Account ID to be deleted
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Enter the beneficiary ID to delete this account ID
    beneficiary: BeneficiaryAccount,
}

impl DeleteAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let beneficiary_id: near_primitives::types::AccountId =
            self.beneficiary.beneficiary_account_id.clone().into();
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::DeleteAccount(
                near_primitives::transaction::DeleteAccountAction { beneficiary_id },
            )],
        };
        match crate::transaction_signature_options::sign_with(
            self.beneficiary.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.beneficiary.network_config.get_network_config(config),
            ),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct BeneficiaryAccount {
    ///Specify a beneficiary
    beneficiary_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}
