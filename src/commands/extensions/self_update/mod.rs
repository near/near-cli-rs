struct DownloadDirs {
    tmp_dir: tempfile::TempDir,
    install_path: std::path::PathBuf,
    archive_path: std::path::PathBuf,
    folder_path: std::path::PathBuf,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    pub async fn process(&self) -> crate::CliResult {
        let self_clone = self.clone();

        tokio::task::spawn_blocking(move || -> crate::CliResult {
            println!(
                "Welcome to NEAR!\n
This will download and install the official release of near-cli-rs."
            );

            let releases = self_update::backends::github::ReleaseList::configure()
                .repo_owner("near")
                .repo_name("near-cli-rs")
                .build()
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to build release list from github: {:?}",
                        err
                    ))
                })?
                .fetch()
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to fetch release list from github: {:?}",
                        err
                    ))
                })?;

            let current_version = clap::crate_version!()
                .parse::<crate::types::version::Version>()
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to parse current version of near-cli-rs as Version: {:?}",
                        err
                    ))
                })?;
            let latest_release_version = releases[0]
                .version
                .parse::<crate::types::version::Version>()
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to parse current version of near-cli-rs as Version: {:?}",
                        err
                    ))
                })?;

            if current_version >= latest_release_version {
                println!("\ninfo: near-cli-rs is already up to date\n");
            } else {
                self_clone.download_release(&releases[0])?;

                println!("\nnear-cli-rs is installed now. Great!\n");
            }

            Ok(())
        })
        .await?
    }

    fn download_release(&self, release: &self_update::update::Release) -> crate::CliResult {
        let install_path = match std::env::current_exe() {
            Ok(path) => path,
            Err(err) => {
                return Err(color_eyre::Report::msg(format!(
                    "Failed to get current near-cli location: {:?}",
                    err
                )))
            }
        };

        println!(
            "near-cli-rs binary will be installed into the current near-cli-rs location:\n\n{}\n",
            install_path.display()
        );

        let triplet = self_update::get_target();
        println!("info: default host triple is {}", triplet);

        let unwrapped_asset;

        if let Some(asset) = release.asset_for(triplet, None) {
            unwrapped_asset = asset;
            println!("info: found release for {}", triplet);
        } else {
            return Err(color_eyre::Report::msg(format!(
                "Failed to find release for {}",
                triplet,
            )));
        }

        let tmp_dir = tempfile::Builder::new()
            .prefix("near-cli")
            .tempdir_in(std::env::current_dir().unwrap())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to create temporary directory: {:?}", err))
            })?;

        let archive_path = std::path::Path::new(&tmp_dir.path()).join(&unwrapped_asset.name);
        let folder_path = std::path::Path::new(&tmp_dir.path())
            .join(unwrapped_asset.name.split(".tar").collect::<Vec<_>>()[0]);

        let tmp_archive = std::fs::File::create(&archive_path).map_err(|err| {
            color_eyre::Report::msg(format!("Failed to create path to an archive: {:?}", err))
        })?;

        println!("info: downloading {}", unwrapped_asset.name);
        self_update::Download::from_url(&unwrapped_asset.download_url)
            .show_progress(true)
            .set_header(
                reqwest::header::ACCEPT,
                "application/octet-stream".parse().unwrap(),
            )
            .download_to(&tmp_archive)
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to download latest release from GitHub: {:?}",
                    err
                ))
            })?;

        self.unpack_archive(
            &unwrapped_asset,
            &DownloadDirs {
                tmp_dir,
                install_path,
                archive_path,
                folder_path,
            },
        )?;

        Ok(())
    }

    fn unpack_archive(
        &self,
        asset: &self_update::update::ReleaseAsset,
        download_dirs: &DownloadDirs,
    ) -> crate::CliResult {
        println!("\ninfo: unpacking {} archive", asset.name);
        let tar_gz = flate2::read::GzDecoder::new(
            std::fs::File::open(&download_dirs.archive_path).map_err(|err| {
                color_eyre::Report::msg(format!("Failed to open archive path: {:?}", err))
            })?,
        );
        let mut tar = tar::Archive::new(tar_gz);
        tar.unpack(download_dirs.tmp_dir.path()).map_err(|err| {
            color_eyre::Report::msg(format!("Failed to unpack archive: {:?}", err))
        })?;

        println!(
            "info: moving near-cli binary to {}",
            download_dirs.install_path.display()
        );
        std::fs::copy(
            download_dirs.folder_path.join("near-cli"),
            &download_dirs.install_path,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to copy near-cli binary to {}: {:?}",
                download_dirs.install_path.display(),
                err
            ))
        })?;

        Ok(())
    }
}
