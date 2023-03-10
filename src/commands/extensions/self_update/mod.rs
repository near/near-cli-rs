#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    pub async fn process(&self) -> crate::CliResult {
        tokio::task::spawn_blocking(move || -> crate::CliResult {
            let status = self_update::backends::github::Update::configure()
                .repo_owner("near")
                .repo_name("near-cli-rs")
                .bin_name("near-cli")
                .show_download_progress(true)
                .target("x86_64-apple-darwin")
                .current_version(self_update::cargo_crate_version!())
                .build()
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to build self_update: {:?}", err))
                })?
                .update()
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to update near-cli-rs: {:?}", err))
                })?;

            println!("\nUpdate status: {:?}", status);
            Ok(())
        })
        .await?
    }
}
