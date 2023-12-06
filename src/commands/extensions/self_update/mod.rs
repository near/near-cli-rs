#[cfg(windows)]
const BIN_NAME: &str = "near.exe";
#[cfg(not(windows))]
const BIN_NAME: &str = "near";

use color_eyre::{eyre::WrapErr, owo_colors::OwoColorize};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SelfUpdateCommandContext)]
pub struct SelfUpdateCommand;

#[derive(Debug, Clone)]
pub struct SelfUpdateCommandContext;

impl SelfUpdateCommandContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        _scope: &<SelfUpdateCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("near")
            .repo_name("near-cli-rs")
            .bin_path_in_archive(
                format!("near-cli-rs-{}/{}", self_update::get_target(), BIN_NAME).as_str(),
            )
            .bin_name(BIN_NAME)
            .show_download_progress(true)
            .current_version(self_update::cargo_crate_version!())
            .build()
            .wrap_err("Failed to build self_update")?
            .update()
            .wrap_err("Failed to update near CLI")?;
        if let self_update::Status::Updated(release) = status {
            println!(
                "\n{}{}{}\n",
                "Welcome to `near` CLI v".green().bold(),
                release.green().bold(),
                "!".green().bold()
            );
            println!("Report any bugs:\n");
            println!("\thttps://github.com/near/near-cli-rs/issues\n");
            println!("What's new:\n");
            println!(
                "\t{}{}\n",
                "https://github.com/near/near-cli-rs/releases/tag/v".truecolor(0, 160, 150),
                release.truecolor(0, 160, 150)
            );
        }

        Ok(Self)
    }
}

pub fn get_latest_version() -> color_eyre::eyre::Result<String> {
    Ok(self_update::backends::github::Update::configure()
        .repo_owner("near")
        .repo_name("near-cli-rs")
        .bin_name("near")
        .current_version(self_update::cargo_crate_version!())
        .build()
        .wrap_err("Failed to build self_update")?
        .get_latest_release()
        .wrap_err("Failed to get latest release")?
        .version)
}
