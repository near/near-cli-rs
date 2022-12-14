#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct LoginFromWebWallet {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network::Network,
}

impl LoginFromWebWallet {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        let mut url: url::Url = network_config.wallet_url.join("login/")?;
        url.query_pairs_mut()
            .append_pair("title", "NEAR CLI")
            .append_pair("public_key", &key_pair_properties.public_key_str);
        // Use `success_url` once capture mode is implemented
        //.append_pair("success_url", "http://127.0.0.1:8080");
        println!(
            "If your browser doesn't automatically open, please visit this URL:\n {}\n",
            &url.as_str()
        );
        // url.open();
        open::that(url.as_ref()).ok();

        let error_message = format!("\nIt is currently not possible to verify the account access key.\nYou may not be logged in to {} or you may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n", &url.as_str());
        super::login(
            network_config,
            config.credentials_home_dir,
            key_pair_properties,
            error_message,
        )
        .await
    }
}
