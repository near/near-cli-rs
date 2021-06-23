use clap::IntoApp;

/// инструмент для настройки терминала пользователя
#[derive(Debug, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliGenerateShellCompletions {
    #[clap(subcommand)]
    shell_type: CliShellCompletionType,
}

#[derive(Debug, clap::Clap)]
pub enum CliShellCompletionType {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl CliGenerateShellCompletions {
    pub fn process(&self) {
        fn generate_shell_completion<T: clap_generate::Generator>() {
            let mut app = crate::CliArgs::into_app();
            let app_name = app.get_name().to_owned();
            clap_generate::generate::<T, _>(&mut app, &app_name, &mut std::io::stdout());
        }

        use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
        match self.shell_type {
            CliShellCompletionType::Bash => generate_shell_completion::<Bash>(),
            CliShellCompletionType::Elvish => generate_shell_completion::<Elvish>(),
            CliShellCompletionType::Fish => generate_shell_completion::<Fish>(),
            CliShellCompletionType::PowerShell => generate_shell_completion::<PowerShell>(),
            CliShellCompletionType::Zsh => generate_shell_completion::<Zsh>(),
        }
    }
}
