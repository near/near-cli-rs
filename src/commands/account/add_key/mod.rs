use inquire::{CustomType, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod access_key_type;
mod autogenerate_new_keypair;
#[cfg(feature = "ledger")]
mod use_ledger;
mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = AddKeyCommandContext)]
pub struct AddKeyCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account should You add an access key to?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    permission: AccessKeyPermission,
}

#[derive(Debug, Clone)]
pub struct AddKeyCommandContext {
    global_context: crate::GlobalContext,
    owner_account_id: crate::types::account_id::AccountId,
}

impl AddKeyCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<AddKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone(),
        })
    }
}

impl AddKeyCommand {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let used_account_list =
            crate::common::get_used_account_list(&context.config.credentials_home_dir)?;
        if used_account_list.is_empty() {
            let account_id: crate::types::account_id::AccountId =
                CustomType::new("Which account should You add an access key to?").prompt()?;
            crate::common::update_used_account_list(
                &context.config.credentials_home_dir,
                account_id.clone().into(),
            )?;
            return Ok(Some(account_id));
        }

        #[derive(derive_more::Display)]
        enum ConfirmOptions {
            #[display(fmt = "I want to manually enter an account.")]
            ManuallyEnter,
            #[display(fmt = "I want to select from a list of accounts.")]
            Select,
        }
        let select_choose_input = Select::new(
            "\nDo you want to enter an account or choose from a list of accounts?",
            vec![ConfirmOptions::ManuallyEnter, ConfirmOptions::Select],
        )
        .prompt()?;
        if let ConfirmOptions::ManuallyEnter = select_choose_input {
            let account_id: crate::types::account_id::AccountId =
                CustomType::new("Which account should You add an access key to?").prompt()?;
            crate::common::update_used_account_list(
                &context.config.credentials_home_dir,
                account_id.clone().into(),
            )?;
            return Ok(Some(account_id));
        }

        let variants = used_account_list
            .iter()
            .map(|account| &account.account_id)
            .collect();
        let select_submit =
            Select::new("Which account should You add an access key to?", variants).prompt();
        match select_submit {
            Ok(account_id) => {
                crate::common::update_used_account_list(
                    &context.config.credentials_home_dir,
                    account_id.clone(),
                )?;

                Ok(Some(account_id.clone().into()))
            }
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = AddKeyCommandContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a permission that you want to add to the access key:
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(
        message = "grant-full-access           - A permission with full access"
    ))]
    /// Provide data for a full access key
    GrantFullAccess(self::access_key_type::FullAccessType),
    #[strum_discriminants(strum(
        message = "grant-function-call-access  - A permission with function call"
    ))]
    /// Provide data for a function-call access key
    GrantFunctionCallAccess(self::access_key_type::FunctionCallType),
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = self::access_key_type::AccessTypeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Add an access key for this account:
pub enum AccessKeyMode {
    #[strum_discriminants(strum(
        message = "autogenerate-new-keypair          - Automatically generate a key pair"
    ))]
    /// Automatically generate a key pair
    AutogenerateNewKeypair(self::autogenerate_new_keypair::GenerateKeypair),
    #[strum_discriminants(strum(
        message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    ))]
    /// Use the provided seed phrase manually
    UseManuallyProvidedSeedPhrase(
        self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    /// Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(message = "use-ledger                        - Use a ledger"))]
    /// Use the Ledger Hadware wallet
    UseLedger(self::use_ledger::AddLedgerKeyAction),
}
