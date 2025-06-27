use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromKeychainContext)]
pub struct PublicKeyFromKeychain {
    #[interactive_clap(skip_default_input_arg)]
    /// For which account do you need to view the public key?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct PublicKeyFromKeychainContext(crate::network::NetworkContext);

impl PublicKeyFromKeychainContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = scope.owner_account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    if previous_context.offline {
                        eprintln!(
                            "\nThe signer's public key cannot be verified and retrieved offline."
                        );
                        return Ok(());
                    }
                    let service_name = std::borrow::Cow::Owned(format!(
                        "near-{}-{}",
                        network_config.network_name, &account_id
                    ));

                    let password = {
                        let access_key_list = network_config
                            .json_rpc_client()
                            .blocking_call_view_access_key_list(
                                &account_id.clone().into(),
                                near_primitives::types::Finality::Final.into(),
                            )
                            .wrap_err_with(|| {
                                format!("Failed to fetch access key list for {account_id}")
                            })?
                            .access_key_list_view()?;

                        let res = access_key_list
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
                                let keyring = keyring::Entry::new(
                                    &service_name,
                                    &format!("{account_id}:{public_key}"),
                                )
                                .ok()?;
                                keyring.get_password().ok()
                            });

                        match res {
                            Some(password) => password,
                            None => {
                                // no access keys found
                                eprintln!("\nNo access keys found in keychain",);
                                return Ok(());
                            }
                        }
                    };

                    let account_key_pair: crate::transaction_signature_options::AccountKeyPair =
                        serde_json::from_str(&password).wrap_err("Error reading data")?;

                    if let crate::Verbosity::Quiet = previous_context.verbosity {
                        println!("{}", account_key_pair.public_key);
                    } else {
                        eprintln!("\nPublic key (printed to stdout): ");
                        println!("{}", account_key_pair.public_key);
                    }

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

impl From<PublicKeyFromKeychainContext> for crate::network::NetworkContext {
    fn from(item: PublicKeyFromKeychainContext) -> Self {
        item.0
    }
}
