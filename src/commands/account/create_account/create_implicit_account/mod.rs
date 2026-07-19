use inquire::CustomType;
use std::io::Write;
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
    /// Enter the file path where to save the implicit account:
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
        Ok(Some(
            CustomType::new("Enter the file path where to save the implicit account:")
                .with_starting_input(&format!(
                    "{}/implicit",
                    context.config.credentials_home_dir.to_string_lossy()
                ))
                .prompt()?,
        ))
    }
}

pub type OnAfterGettingFolderPathCallback =
    std::sync::Arc<dyn Fn(&std::path::PathBuf) -> crate::CliResult>;

#[derive(Clone)]
pub struct SaveImplicitAccountContext {
    config: crate::config::Config,
    on_after_getting_folder_path_callback: OnAfterGettingFolderPathCallback,
}

fn write_secret_key_file(file_path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        options.mode(0o600);
        let mut file = options.open(file_path)?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(contents)
    }

    #[cfg(not(unix))]
    options.open(file_path)?.write_all(contents)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    #[test]
    fn secret_key_file_is_owner_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("implicit-account.json");

        super::write_secret_key_file(&file_path, b"first").unwrap();
        assert_eq!(file_path.metadata().unwrap().mode() & 0o777, 0o600);

        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        super::write_secret_key_file(&file_path, b"replacement").unwrap();

        assert_eq!(std::fs::read(&file_path).unwrap(), b"replacement");
        assert_eq!(file_path.metadata().unwrap().mode() & 0o777, 0o600);
    }
}
