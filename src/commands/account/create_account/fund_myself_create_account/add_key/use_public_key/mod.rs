#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
pub struct AddAccessKeyAction {
    ///Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

impl AddAccessKeyAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        let account_properties = super::super::super::AccountProperties {
            public_key: self.public_key.clone().into(),
            ..account_properties
        };
        let storage_properties = None;
        self.sign_as
            .process(config, account_properties, storage_properties)
            .await
    }
}
