use inquire::Text;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod use_auto_generation;
#[cfg(feature = "ledger")]
mod use_ledger;
mod use_seed_phrase;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImplicitAccount {
    #[interactive_clap(subcommand)]
    mode: Mode,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a mode to create an implicit account:
pub enum Mode {
    #[strum_discriminants(strum(
        message = "use-auto-generation  - Use auto-generation to create an implicit account"
    ))]
    /// Use auto-generation to create an implicit account
    UseAutoGeneration(self::use_auto_generation::SaveWithUseAutoGeneration),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "use-ledger           - Use ledger to create an implicit account"
    ))]
    /// Use ledger to create an implicit account
    UseLedger(self::use_ledger::SaveWithLedger),
    #[strum_discriminants(strum(
        message = "use-seed-phrase      - Use seed phrase to create an implicit account"
    ))]
    /// Use seed phrase to create an implicit account
    UseSeedPhrase(self::use_seed_phrase::SaveWithSeedPhrase),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SaveImplicitAccountContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SaveToFolder {
    #[interactive_clap(skip_default_input_arg)]
    /// Where to save the implicit account file?
    folder_path: crate::types::path_buf::PathBuf,
}

impl interactive_clap::FromCli for SaveToFolder {
    type FromCliContext = SaveImplicitAccountContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        if clap_variant.folder_path.is_none() {
            clap_variant.folder_path = match Self::input_folder_path(&context) {
                Ok(Some(folder_path)) => Some(folder_path),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let folder_path = clap_variant.folder_path.clone().expect("Unexpected error");

        match (context.on_after_getting_folder_path_callback)(&folder_path.into()) {
            Ok(_) => interactive_clap::ResultFromCli::Ok(clap_variant),
            Err(err) => interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        }
    }
}

impl SaveToFolder {
    fn input_folder_path(
        context: &SaveImplicitAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        eprintln!();
        let input_folder_path: String = Text::new("Where to save the implicit account file?")
            .with_initial_value(
                format!(
                    "{}/implicit",
                    context.config.credentials_home_dir.to_string_lossy()
                )
                .as_str(),
            )
            .prompt()?;
        let folder_path = shellexpand::tilde(&input_folder_path).as_ref().parse()?;
        Ok(Some(folder_path))
    }
}

pub type OnAfterGettingFolderPathCallback =
    std::sync::Arc<dyn Fn(&std::path::PathBuf) -> crate::CliResult>;

#[derive(Clone)]
pub struct SaveImplicitAccountContext {
    config: crate::config::Config,
    on_after_getting_folder_path_callback: OnAfterGettingFolderPathCallback,
}
