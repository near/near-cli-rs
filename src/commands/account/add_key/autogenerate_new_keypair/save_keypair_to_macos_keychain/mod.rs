#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
pub struct SaveKeypairToMacosKeychain {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}



// #[derive(Debug, Clone)]
// pub struct GenerateKeypairContext {
//     config: crate::config::Config,
//     signer_account_id: near_primitives::types::AccountId,
//     permission: near_primitives::account::AccessKeyPermission,
//     key_pair_properties: crate::common::KeyPairProperties,
//     public_key: near_crypto::PublicKey
// }

// impl GenerateKeypairContext {
//     pub fn from_previous_context(
//         previous_context: super::access_key_type::AccessTypeContext,
//         scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
//     ) -> color_eyre::eyre::Result<Self> {
//         let key_pair_properties: crate::common::KeyPairProperties = tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(crate::common::generate_keypair())?;
//         let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
//         Ok(Self {
//             config: previous_context.config,
//             signer_account_id: previous_context.signer_account_id,
//             permission: previous_context.permission,
//             key_pair_properties,
//             public_key
//         })
//     }
// }

// impl From<AddAccessKeyActionContext> for crate::commands::ActionContext {
//     fn from(item: AddAccessKeyActionContext) -> Self {
//         Self {
//             config: item.config,
//             signer_account_id: item.signer_account_id.clone(),
//             receiver_account_id: item.signer_account_id,
//             actions: vec![near_primitives::transaction::Action::AddKey(
//                 near_primitives::transaction::AddKeyAction {
//                     public_key: item.public_key.into(),
//                     access_key: near_primitives::account::AccessKey {
//                         nonce: 0,
//                         permission: item.permission,
//                     },
//                 },
//             )],
//         }
//     }
// }





impl SaveKeypairToMacosKeychain {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
        crate::common::save_access_key_to_macos_keychain(
            network_config,
            &key_pair_properties_buf,
            &key_pair_properties.public_key_str,
            &prepopulated_unsigned_transaction.receiver_id,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;
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
