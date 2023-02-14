use std::{collections::HashMap, io::Write};

struct DownloadDirs {
    tmp_dir: tempfile::TempDir,
    bin_dir: std::path::PathBuf,
    archive_path: std::path::PathBuf,
    folder_path: std::path::PathBuf,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    pub async fn process(&self) -> crate::CliResult {
        let self_clone = self.clone();

        tokio::task::spawn_blocking(move || {
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

            if current_version == latest_release_version {
                println!("\ninfo: near-cli-rs is already up to date\n");
                Ok(())
            } else {
                self_clone.download_release(&releases[0])?;
                self_clone.export_path("$HOME/.local/bin")?;

                println!("\nnear-cli-rs is installed now. Great!\n");

                Ok(())
            }
        })
        .await?
    }

    fn download_release(
        &self,
        release: &self_update::update::Release,
    ) -> color_eyre::eyre::Result<self_update::update::ReleaseAsset> {
        let home_dir = dirs::home_dir().expect("Failed to get home directory path");
        let bin_dir = home_dir.join(".local/bin");

        println!("near-cli-rs binary will be installed into the local bin home directory, located at:\n\n{}\n", bin_dir.display());

        let mut compatible_triplets = HashMap::new();
        compatible_triplets.insert("aarch64-apple-darwin", "x86_64-apple-darwin");

        let triplet = self_update::get_target();
        println!("info: default host triple is {}", triplet);

        let asset = if compatible_triplets.contains_key(triplet) {
            println!(
                "info: could not find release for {}, trying to download {} instead",
                triplet,
                compatible_triplets.get(triplet).unwrap()
            );

            release
                .asset_for(compatible_triplets.get(triplet).unwrap())
                .unwrap()
        } else {
            release.asset_for(triplet).unwrap()
        };

        let tmp_dir = tempfile::Builder::new()
            .prefix("near-cli")
            .tempdir_in(std::env::current_dir().unwrap())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to create temporary directory: {:?}", err))
            })?;

        let archive_path = std::path::Path::new(&tmp_dir.path()).join(&asset.name);
        let folder_path = std::path::Path::new(&tmp_dir.path())
            .join(asset.name.split(".tar").collect::<Vec<_>>()[0]);

        let tmp_archive = std::fs::File::create(&archive_path).map_err(|err| {
            color_eyre::Report::msg(format!("Failed to create path to an archive: {:?}", err))
        })?;

        println!("info: downloading {}", asset.name);
        self_update::Download::from_url(&asset.download_url)
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
            &asset,
            DownloadDirs {
                tmp_dir,
                bin_dir,
                archive_path,
                folder_path,
            },
        )?;

        Ok(asset)
    }

    fn unpack_archive(
        &self,
        asset: &self_update::update::ReleaseAsset,
        download_dirs: DownloadDirs,
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

        println!("info: moving near-cli binary to $HOME/.local/bin");
        std::fs::copy(
            download_dirs.folder_path.join("near-cli"),
            download_dirs.bin_dir.join("near-cli"),
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to copy near-cli binary to $HOME/.local/bin: {:?}",
                err
            ))
        })?;

        Ok(())
    }

    fn export_path(&self, path: &str) -> crate::CliResult {
        let mut export = true;

        for path in env!("PATH").split(':') {
            if std::path::Path::new(path).eq(&dirs::home_dir()
                .expect("Failed to get path to home directory")
                .join(".local/bin"))
            {
                export = false;
                break;
            }
        }

        if export {
            println!(
                "To get started you may need to restart your current shell.
This would reload your PATH environment variable to include
local's bin directory ($HOME/.local/bin)."
            );

            let home_dir = dirs::home_dir().unwrap();
            let shell = env!("SHELL").split('/').last().unwrap_or("fruit");

            let profile_file = match shell {
                "bash" => ".bash_profile",
                "zsh" => ".zshrc",
                "fish" => ".config/fish/config.fish",
                _ => ".bash_profile",
            };
            let profile_path = home_dir.join(profile_file);

            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&profile_path)
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to open file `{}`: {:?}",
                        profile_path.display(),
                        err
                    ))
                })?;

            let line = match shell {
                "bash" => format!("export PATH={}:{}\n", path, "$PATH"),
                "zsh" => format!("export PATH={}:{}\n", path, "$PATH"),
                "fish" => format!("set -gx PATH {} $PATH\n", path),
                _ => format!("export PATH={}:{}\n", path, "$PATH"),
            };

            file.write(line.as_bytes()).map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to write to file `{}`: {:?}",
                    profile_path.display(),
                    err
                ))
            })?;
            println!("info: PATH was added to {}", profile_path.display());
        } else {
            println!("info: $HOME/.local/bin is already in PATH variable");
        }

        Ok(())
    }
}
