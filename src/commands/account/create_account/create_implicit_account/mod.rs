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
#[interactive_clap(input_context = SaveImplicitAccountContext)]
#[interactive_clap(output_context = SaveToFolderContext)]
pub struct SaveToFolder {
    #[interactive_clap(skip_default_input_arg)]
    /// Where to save the implicit account file?
    folder_path: crate::types::path_buf::PathBuf,
}

#[derive(Clone)]
struct SaveToFolderContext;

impl SaveToFolderContext {
    pub fn from_previous_context(
        previous_context: SaveImplicitAccountContext,
        scope: &<SaveToFolder as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        (previous_context.on_after_getting_folder_path_callback)(
            &scope.folder_path.clone().into(),
        )?;
        Ok(Self)
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
