#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    pub async fn process(&self) -> crate::CliResult {
        tokio::task::spawn_blocking(move || -> crate::CliResult {
            self_update::backends::github::Update::configure()
                .repo_owner("near")
                .repo_name("near-cli-rs")
                .bin_path_in_archive(
                    format!(
                        "near-cli-{}-{}/near-cli",
                        self_update::cargo_crate_version!(),
                        self_update::get_target()
                    )
                    .as_str(),
                )
                .bin_name("near-cli")
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
            Ok(())
        })
        .await?
    }
}
