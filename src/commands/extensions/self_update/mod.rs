use reqwest::header::USER_AGENT;
use tokio::time::{sleep, Duration};

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    pub async fn process(&self) -> crate::CliResult {
        let github_releases_api = "https://api.github.com/repos/near/near-cli-rs/releases/latest";

        let current_version = clap::crate_version!()
            .parse::<crate::types::version::Version>()
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to parse current version of near-cli-rs as Version: {:?}",
                    err
                ))
            })?;
        let latest_release_version = self.get_new_version(github_releases_api).await?;

        println!(
            "Updating\n{} -> {}",
            current_version.to_string(),
            latest_release_version.to_string()
        );
        self.download_release(current_version).await?;

        // if current_version == latest_release_version {
        //     println!("You are already up to date");
        // } else if current_version < latest_release_version {
        //     println!(
        //         "Updating\n{} -> {}",
        //         current_version.to_string(),
        //         latest_release_version.to_string()
        //     );
        //     self.download_release(current_version).await?;
        // }

        Ok(())
    }

    async fn get_new_version(
        &self,
        api: &str,
    ) -> color_eyre::eyre::Result<crate::types::version::Version> {
        let client = reqwest::Client::new();

        for _ in 0..10 {
            let response = client
                .get(api)
                .header(USER_AGENT, "Just Random")
                .send()
                .await
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to send request to get latest git release version: {:?}",
                        err
                    ))
                })?;

            let response_json = json::parse(
                response
                    .text()
                    .await
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to parse request response as a text: {:?}",
                            err
                        ))
                    })?
                    .as_str(),
            )
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to parse request response as a json: {:?}",
                    err
                ))
            })?;

            match response_json["name"].as_str() {
                Some(version) => {
                    return Ok(version
                        .trim()
                        .parse::<crate::types::version::Version>()
                        .map_err(|err| {
                            color_eyre::Report::msg(format!(
                                "Failed to parse current version of near-cli-rs as Version: {:?}",
                                err
                            ))
                        })?);
                }
                None => {
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }

        Err(color_eyre::Report::msg(
            "Failed to get last release version from github".to_string(),
        ))
    }

    async fn download_release(&self, version: crate::types::version::Version) -> crate::CliResult {
        let home_dir = dirs::home_dir().expect("Failed to get home directory path");
        let bin_dir = home_dir.join(".local/bin");

        if !bin_dir.is_dir() {
            std::fs::create_dir(bin_dir.clone()).map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to create ~/.local/bin directory: {:?}",
                    err
                ))
            })?;
            println!("Created ~/.local/bin");
        }

        // let filename = format!(
        //     "near-cli-{}-{}.tar.gz",
        //     version.to_string(),
        //     current_platform::CURRENT_PLATFORM
        // );

        let hardcoded_filename = format!(
            "near-cli-{}-x86_64-apple-darwin.tar.gz",
            version.to_string()
        );
        let hardcoded_unpacked_filename =
            format!("near-cli-{}-x86_64-apple-darwin", version.to_string());

        let response = reqwest::get(format!(
            "https://github.com/near/near-cli-rs/releases/download/{}/{}",
            version.to_string(),
            hardcoded_filename
        ))
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to send request to download latest near-cli-rs release: {:?}",
                err
            ))
        })?;

        if response.status().is_success() {
            let data = response.bytes().await?;

            let temp_dir = tempfile::TempDir::new().map_err(|err| {
                color_eyre::Report::msg(format!("Failed to create temporary directory: {:?}", err))
            })?;
            let file_path = Path::new(&temp_dir.path()).join(hardcoded_filename);

            fs::write(&file_path, data)?;

            let tar_gz = flate2::read::GzDecoder::new(fs::File::open(&file_path)?);
            let mut tar = tar::Archive::new(tar_gz);
            tar.unpack(&temp_dir.path())?;

            let unpacked_file_path = Path::new(&temp_dir.path()).join(hardcoded_unpacked_filename);

            fs::copy(
                unpacked_file_path.join("near-cli"),
                bin_dir.join("near-cli"),
            )
            .unwrap();

            println!("{}", bin_dir.join("near-cli").display());
        } else {
            return Err(color_eyre::Report::msg(
                "Failed to download new version of near-cli-rs".to_string(),
            ));
        }

        Ok(())
    }
}
