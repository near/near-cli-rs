use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliServer {}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        Server { connection_config }
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        let status = near_jsonrpc_client::new_client(self.connection_config.rpc_url().as_str())
            .status()
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        println!(
            "recent block hash: {:?}",
            status.sync_info.latest_block_hash
        );
        Ok(())
    }
}
