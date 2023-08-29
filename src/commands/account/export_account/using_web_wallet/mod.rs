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
        let account_id = previous_context.account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    #[cfg(target_os = "macos")]
                    {
                        if let Ok(account_key_pair) =
                            get_account_key_pair_from_macos_keychain(network_config, &account_id)
                        {
                            return auto_import_secret_key(
                                network_config,
                                &account_id,
                                &account_key_pair.private_key,
                            );
                        }
                    }

                    let account_key_pair = get_account_key_pair_from_keychain(
                        network_config,
                        &account_id,
                        &config.credentials_home_dir,
                    )?;
                    auto_import_secret_key(
                        network_config,
                        &account_id,
                        &account_key_pair.private_key,
                    )
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.account_id],
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
pub fn get_account_key_pair_from_macos_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<crate::transaction_signature_options::AccountKeyPair> {
    let password_list = get_password_list_from_macos_keychain(network_config, account_id)?;
    for password in password_list {
        let account_key_pair = serde_json::from_slice(password.as_ref());
        if let Ok(key_pair) = account_key_pair {
            return Ok(key_pair);
        }
    }
    Err(color_eyre::eyre::Report::msg("Error reading data"))
}

#[cfg(target_os = "macos")]
pub fn get_password_list_from_macos_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Vec<security_framework::os::macos::passwords::SecKeychainItemPassword>>
{
    let keychain = security_framework::os::macos::keychain::SecKeychain::default()
        .wrap_err("Failed to open keychain")?;

    let service_name: std::borrow::Cow<'_, str> = std::borrow::Cow::Owned(format!(
        "near-{}-{}",
        network_config.network_name,
        account_id.as_str()
    ));
    let password_list = {
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
            .filter_map(|key| {
                let password = keychain.find_generic_password(
                    &service_name,
                    &format!("{}:{}", account_id, key.public_key),
                );
                match password {
                    Ok((pas, _)) => Some(pas),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>()
    };
    Ok(password_list)
}

pub fn get_account_key_pair_from_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::Path,
) -> color_eyre::eyre::Result<crate::transaction_signature_options::AccountKeyPair> {
    let data_path =
        get_account_key_pair_data_path(network_config, account_id, credentials_home_dir)?;
    let data = std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
    let account_key_pair: crate::transaction_signature_options::AccountKeyPair =
        serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &data_path))?;
    Ok(account_key_pair)
}

fn get_account_key_pair_data_path(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::Path,
) -> color_eyre::eyre::Result<std::path::PathBuf> {
    let check_if_seed_phrase_exists = false;
    get_account_properties_data_path(
        network_config,
        account_id,
        credentials_home_dir,
        check_if_seed_phrase_exists,
    )
}

pub fn get_account_properties_data_path(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::Path,
    check_if_seed_phrase_exists: bool,
) -> color_eyre::eyre::Result<std::path::PathBuf> {
    let file_name = format!("{}.json", account_id);
    let mut path = std::path::PathBuf::from(credentials_home_dir);

    let dir_name = network_config.network_name.clone();
    path.push(&dir_name);
    path.push(file_name);

    if path.exists() {
        if !check_if_seed_phrase_exists {
            return Ok(path);
        }
        let data = std::fs::read_to_string(&path).wrap_err("Access key file not found!")?;
        if serde_json::from_str::<crate::common::KeyPairProperties>(&data).is_ok() {
            return Ok(path);
        }
    }

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
    for access_key in access_key_list.keys {
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
                if !check_if_seed_phrase_exists {
                    return Ok(data_path);
                }
                let data =
                    std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
                if serde_json::from_str::<crate::common::KeyPairProperties>(&data).is_ok() {
                    return Ok(data_path);
                } else {
                    return Err(color_eyre::eyre::Report::msg(format!(
                        "There are no master seed phrase in keychain to export for account <{account_id}>."
                    )));
                }
            }
        }
    }
    Err(color_eyre::eyre::Report::msg(format!(
        "There are no access keys in keychain to export for account <{account_id}>."
    )))
}
