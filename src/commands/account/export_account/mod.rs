use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod using_private_key;
mod using_seed_phrase;
mod using_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ExportAccountContext)]
pub struct ExportAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account ID should be exported?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    import_account_actions: ExportAccountActions,
}

#[derive(Debug, Clone)]
pub struct ExportAccountContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl ExportAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ExportAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl ExportAccount {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        Ok(Some(
            crate::common::input_account_id_from_used_account_list(
                &context.config.credentials_home_dir,
                "Which account ID should be exported?",
                true,
            )?,
        ))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ExportAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to export the account?
pub enum ExportAccountActions {
    #[strum_discriminants(strum(
        message = "using-web-wallet          - Export existing account using NEAR Wallet"
    ))]
    /// Export existing account using NEAR Wallet
    UsingWebWallet(self::using_web_wallet::ExportAccountFromWebWallet),
    #[strum_discriminants(strum(
        message = "using-seed-phrase         - Export existing account using a seed phrase"
    ))]
    /// Export existing account using a seed phrase
    UsingSeedPhrase(self::using_seed_phrase::ExportAccountFromSeedPhrase),
    #[strum_discriminants(strum(
        message = "using-private-key         - Export existing account using a private key"
    ))]
    /// Export existing account using a private key
    UsingPrivateKey(self::using_private_key::ExportAccountFromPrivateKey),
}
