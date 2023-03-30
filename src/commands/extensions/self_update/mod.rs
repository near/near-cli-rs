#[cfg(windows)]
const BIN_NAME: &str = "near-cli.exe";
#[cfg(not(windows))]
const BIN_NAME: &str = "near-cli";

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
        self_update::backends::github::Update::configure()
            .repo_owner("near")
            .repo_name("near-cli-rs")
            .bin_path_in_archive(
                format!(
                    "near-cli-{}-{}/{}",
                    get_latest_version()?,
                    self_update::get_target(),
                    BIN_NAME
                )
                .as_str(),
            )
            .bin_name(BIN_NAME)
            .show_download_progress(true)
            .current_version(self_update::cargo_crate_version!())
            .build()
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to build self_update: {:?}", err))
            })?
            .update()
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to update near-cli-rs: {:?}", err))
            })?;
        Ok(Self)
    }
}

pub fn get_latest_version() -> color_eyre::eyre::Result<String> {
    Ok(self_update::backends::github::Update::configure()
        .repo_owner("near")
        .repo_name("near-cli-rs")
        .bin_name("near-cli")
        .current_version(self_update::cargo_crate_version!())
        .build()
        .map_err(|err| color_eyre::Report::msg(format!("Failed to build self_update: {:?}", err)))?
        .get_latest_release()
        .map_err(|err| color_eyre::Report::msg(format!("Failed to get latest release: {:?}", err)))?
        .version)
}
