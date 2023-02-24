#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = crate::commands::account::create_account::CreateAccountContext)]
pub struct AddPublicKeyAction {
    /// Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[derive(Debug, Clone)]
pub struct AddPublicKeyActionContext(
    crate::commands::account::create_account::CreateAccountContext,
);

impl AddPublicKeyActionContext {
    pub fn from_previous_context(
        previous_context: crate::commands::account::create_account::CreateAccountContext,
        scope: &<AddPublicKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_properties = super::super::super::AccountProperties {
            public_key: scope.public_key.clone().into(),
            ..previous_context.account_properties
        };

        Ok(Self(
            crate::commands::account::create_account::CreateAccountContext {
                account_properties,
                ..previous_context
            },
        ))
    }
}

impl From<AddPublicKeyActionContext>
    for crate::commands::account::create_account::CreateAccountContext
{
    fn from(item: AddPublicKeyActionContext) -> Self {
        item.0
    }
}

impl AddPublicKeyAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        // let account_properties = super::super::super::AccountProperties {
        //     public_key: self.public_key.clone().into(),
        //     ..account_properties
        // };
        // let storage_properties = None;
        // self.sign_as
        //     .process(config, account_properties, storage_properties)
        //     .await
        Ok(())
    }
}
