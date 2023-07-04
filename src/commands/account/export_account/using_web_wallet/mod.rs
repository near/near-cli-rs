use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ExportAccountContext)]
#[interactive_clap(output_context = ExportAccountFromWebWalletContext)]
pub struct ExportAccountFromWebWallet {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ExportAccountFromWebWalletContext(crate::network::NetworkContext);

impl ExportAccountFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: super::ExportAccountContext,
        _scope: &<ExportAccountFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.global_context.config.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let mut account_key_pair: Option<
                        crate::transaction_signature_options::AccountKeyPair,
                    > = None;

                    #[cfg(target_os = "macos")]
                    {
                        account_key_pair = account_key_pair_from_macos_keychain(
                            network_config,
                            &previous_context.account_id,
                        )?
                    };

                    if let Some(account_key_pair) = account_key_pair {
                        auto_import_secret_key(
                            network_config,
                            &previous_context.account_id,
                            &account_key_pair.private_key,
                        )
                    } else if let Some(account_key_pair) = account_key_pair_from_keychain(
                        network_config,
                        &previous_context.account_id,
                        &config.credentials_home_dir,
                    )? {
                        auto_import_secret_key(
                            network_config,
                            &previous_context.account_id,
                            &account_key_pair.private_key,
                        )
                    } else {
                        Err(color_eyre::eyre::Report::msg(format!("The macOS keychain or keychain is missing access keys for the {} account.", previous_context.account_id)))
                    }
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<ExportAccountFromWebWalletContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromWebWalletContext) -> Self {
        item.0
    }
}

fn auto_import_secret_key(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    private_key: &near_crypto::SecretKey,
) -> crate::CliResult {
    let mut url: url::Url = network_config.wallet_url.join("auto-import-secret-key")?;
    let fragment = format!("{}/{}", account_id, private_key);
    url.set_fragment(Some(&fragment));
    eprintln!(
        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
        &url.as_str()
    );
    // url.open();
    open::that(url.as_ref()).ok();
    Ok(())
}

#[cfg(target_os = "macos")]
fn account_key_pair_from_macos_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Option<crate::transaction_signature_options::AccountKeyPair>> {
    let keychain = security_framework::os::macos::keychain::SecKeychain::default()
        .wrap_err("Failed to open keychain")?;

    let service_name: std::borrow::Cow<'_, str> = std::borrow::Cow::Owned(format!(
        "near-{}-{}",
        network_config.network_name,
        account_id.as_str()
    ));
    let password = {
        let access_key_list = network_config
            .json_rpc_client()
            .blocking_call_view_access_key_list(
                account_id,
                near_primitives::types::Finality::Final.into(),
            )
            .wrap_err_with(|| format!("Failed to fetch access key list for {}", account_id))?
            .access_key_list_view()?;

        access_key_list
            .keys
            .into_iter()
            .filter(|key| {
                matches!(
                    key.access_key.permission,
                    near_primitives::views::AccessKeyPermissionView::FullAccess
                )
            })
            .map(|key| key.public_key)
            .find_map(|public_key| {
                let (password, _) = keychain
                    .find_generic_password(&service_name, &format!("{}:{}", account_id, public_key))
                    .ok()?;
                Some(password)
            })
    };
    if let Some(password) = password {
        serde_json::from_slice(password.as_ref()).wrap_err("Error reading data")
    } else {
        Ok(None)
    }
}

fn account_key_pair_from_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::PathBuf,
) -> color_eyre::eyre::Result<Option<crate::transaction_signature_options::AccountKeyPair>> {
    let file_name = format!("{}.json", account_id);
    let mut path = std::path::PathBuf::from(credentials_home_dir);

    let data_path: std::path::PathBuf = {
        let dir_name = network_config.network_name.clone();
        path.push(&dir_name);

        path.push(file_name);
        if path.exists() {
            path
        } else {
            let access_key_list = network_config
                .json_rpc_client()
                .blocking_call_view_access_key_list(
                    account_id,
                    near_primitives::types::Finality::Final.into(),
                )
                .wrap_err_with(|| format!("Failed to fetch access KeyList for {}", account_id))?
                .access_key_list_view()?;
            let mut path = std::path::PathBuf::from(credentials_home_dir);
            path.push(dir_name);
            path.push(account_id.to_string());
            let mut data_path = std::path::PathBuf::new();
            'outer: for access_key in access_key_list.keys {
                let account_public_key = access_key.public_key.to_string();
                let is_full_access_key: bool = match &access_key.access_key.permission {
                    near_primitives::views::AccessKeyPermissionView::FullAccess => true,
                    near_primitives::views::AccessKeyPermissionView::FunctionCall {
                        allowance: _,
                        receiver_id: _,
                        method_names: _,
                    } => false,
                };
                let dir = path
                    .read_dir()
                    .wrap_err("There are no access keys found in the keychain for the account.")?;
                for entry in dir.flatten() {
                    // if let Ok(entry) = entry {
                    if entry
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(account_public_key.rsplit(':').next().unwrap())
                        && is_full_access_key
                    {
                        data_path.push(entry.path());
                        break 'outer;
                    }
                    // };
                }
            }
            data_path
        }
    };
    if data_path.exists() {
        let data = std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
        let account_key_pair: crate::transaction_signature_options::AccountKeyPair =
            serde_json::from_str(&data)
                .wrap_err_with(|| format!("Error reading data from file: {:?}", &data_path))?;
        return Ok(Some(account_key_pair));
    }
    Ok(None)
}
