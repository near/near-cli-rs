use color_eyre::eyre::{ContextCompat, WrapErr};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

mod using_private_key;
mod using_seed_phrase;
mod using_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ExportAccountContext)]
pub struct ExportAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account ID should be exported?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    export_account_actions: ExportAccountActions,
}

#[derive(Debug, Clone)]
pub struct ExportAccountContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl ExportAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ExportAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl ExportAccount {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Which account ID should be exported?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ExportAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to export the account?
pub enum ExportAccountActions {
    #[strum_discriminants(strum(
        message = "using-web-wallet          - Export existing account using NEAR Wallet"
    ))]
    /// Export existing account using NEAR Wallet
    UsingWebWallet(self::using_web_wallet::ExportAccountFromWebWallet),
    #[strum_discriminants(strum(
        message = "using-seed-phrase         - Export existing account using a seed phrase"
    ))]
    /// Export existing account using a seed phrase
    UsingSeedPhrase(self::using_seed_phrase::ExportAccountFromSeedPhrase),
    #[strum_discriminants(strum(
        message = "using-private-key         - Export existing account using a private key"
    ))]
    /// Export existing account using a private key
    UsingPrivateKey(self::using_private_key::ExportAccountFromPrivateKey),
}

pub fn get_account_key_pair_from_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<crate::transaction_signature_options::AccountKeyPair> {
    let password = get_password_from_keychain(network_config, account_id)?;
    let account_key_pair = serde_json::from_str(&password);
    account_key_pair.wrap_err("Error reading data")
}

#[tracing::instrument(
    name = "Receiving the account key pair from the keychain ...",
    skip_all
)]
pub fn get_password_from_keychain(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<String> {
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
                let keyring =
                    keyring::Entry::new(&service_name, &format!("{}:{}", account_id, public_key))
                        .ok()?;
                keyring.get_password().ok()
            })
            .wrap_err("No access keys found in keychain")?
    };
    Ok(password)
}

pub fn get_account_key_pair_from_legacy_keychain(
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

#[tracing::instrument(
    name = "Receiving the account key pair from a legacy keychain ...",
    skip_all
)]
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
        let data = std::fs::read_to_string(&path).wrap_err_with(|| {
            format!(
                "Access key file for account <{}> on network <{}> not found!",
                account_id, network_config.network_name
            )
        })?;
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
        let account_public_key = access_key.public_key.to_string().replace(':', "_");
        match &access_key.access_key.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => {}
            near_primitives::views::AccessKeyPermissionView::FunctionCall { .. } => {
                continue;
            }
        }
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
                .contains(&account_public_key)
            {
                data_path.push(entry.path());
                if !check_if_seed_phrase_exists {
                    return Ok(data_path);
                }
                let data =
                    std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
                serde_json::from_str::<crate::common::KeyPairProperties>(&data).wrap_err_with(|| format!(
                        "There are no master seed phrase in keychain to export for account <{account_id}>."
                    ))?;
                return Ok(data_path);
            }
        }
    }
    Err(color_eyre::eyre::Report::msg(format!(
        "There are no access keys in keychain to export for account <{account_id}> on network <{}>.",
        network_config.network_name
    )))
}
