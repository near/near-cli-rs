use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromLegacyKeychainContext)]
pub struct PublicKeyFromKeychain {
    #[interactive_clap(skip_default_input_arg)]
    /// For which account do you need to view the public key?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct PublicKeyFromLegacyKeychainContext(crate::network::NetworkContext);

impl PublicKeyFromLegacyKeychainContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.config.clone();
        let account_id = scope.owner_account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let keychain_folder = config
                        .credentials_home_dir
                        .join(&network_config.network_name);
                    let signer_keychain_folder = keychain_folder.join(account_id.to_string());
                    let signer_access_key_file_path: std::path::PathBuf = {
                        if previous_context.offline {
                            eprintln!(
                                "\nThe signer's public key cannot be verified and retrieved offline."
                            );
                            return Ok(());
                        }
                        if signer_keychain_folder.exists() {
                            let full_access_key_filenames = network_config
                            .json_rpc_client()
                            .blocking_call_view_access_key_list(
                                &account_id.clone().into(),
                                near_primitives::types::Finality::Final.into(),
                            )
                            .wrap_err_with(|| {
                                format!(
                                    "Failed to fetch access KeyList for {account_id}"
                                )
                            })?
                            .access_key_list_view()?
                            .keys
                            .iter()
                            .filter(
                                |access_key_info| match access_key_info.access_key.permission {
                                    near_primitives::views::AccessKeyPermissionView::FullAccess => true,
                                    near_primitives::views::AccessKeyPermissionView::FunctionCall {
                                        ..
                                    } => false,
                                },
                            )
                            .map(|access_key_info| {
                                format!(
                                    "{}.json",
                                    access_key_info.public_key.to_string().replace(":", "_")
                                )
                                .into()
                            })
                            .collect::<std::collections::HashSet<std::ffi::OsString>>();

                            signer_keychain_folder
                            .read_dir()
                            .wrap_err("There are no access keys found in the keychain for the signer account. Import an access key for an account before signing transactions with keychain.")?
                            .filter_map(Result::ok)
                            .find(|entry| full_access_key_filenames.contains(&entry.file_name()))
                            .map(|signer_access_key| signer_access_key.path())
                            .unwrap_or_else(|| keychain_folder.join(format!(
                                "{account_id}.json"
                            )))
                        } else {
                            keychain_folder.join(format!("{account_id}.json"))
                        }
                    };
                    let signer_access_key_json =
                    std::fs::read(&signer_access_key_file_path).wrap_err_with(|| {
                        format!(
                            "Access key file for account <{}> on network <{}> not found! \nSearch location: {:?}",
                            account_id,
                            network_config.network_name, signer_access_key_file_path
                        )
                    })?;
                    let account_key_pair: crate::transaction_signature_options::AccountKeyPair =
                        serde_json::from_slice(&signer_access_key_json).wrap_err_with(|| {
                            format!(
                                "Error reading data from file: {:?}",
                                &signer_access_key_file_path
                            )
                        })?;

                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        previous_context.verbosity
                    {
                        eprint!("Public key (printed to stdout): ");
                    }
                    println!("{}", account_key_pair.public_key);

                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.owner_account_id.clone().into()],
            on_after_getting_network_callback,
        }))
    }
}

impl PublicKeyFromKeychain {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "For which account do you need to view the public key?",
        )
    }
}

impl From<PublicKeyFromLegacyKeychainContext> for crate::network::NetworkContext {
    fn from(item: PublicKeyFromLegacyKeychainContext) -> Self {
        item.0
    }
}
