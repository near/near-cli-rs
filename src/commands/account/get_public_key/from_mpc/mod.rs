use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromMpcContext)]
pub struct PublicKeyFromMpc {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the controllable account address (where public key would've been published)?
    controllable_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(skip_default_input_arg)]
    /// What is the admin account address?
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subcommand)]
    /// What is key type for deriving key?
    mpc_key_type: MpcKeyType,
}

#[derive(Clone)]
pub struct PublicKeyFromMpcContext {
    global_context: crate::GlobalContext,
    controllable_account_id: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
}

impl PublicKeyFromMpcContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromMpc as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if previous_context.offline {
            eprintln!("\nInternet connection is required to derive key from MPC contract!");
            return Err(color_eyre::eyre::eyre!(
                "Internet connection is required to derive key from MPC contract!"
            ));
        }

        Ok(Self {
            global_context: previous_context,
            controllable_account_id: scope.controllable_account_id.clone().into(),
            admin_account_id: scope.admin_account_id.clone().into(),
        })
    }
}

impl PublicKeyFromMpc {
    fn input_controllable_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the controllable account address (where public key would've been published)?",
        )
    }

    fn input_admin_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the admin account address?",
        )
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PublicKeyFromMpcContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What is the key type for derivation (if unsure choose Ed25519)?
pub enum MpcKeyType {
    #[strum_discriminants(strum(message = "ed25519   - use Ed25519 key derivation"))]
    /// Use Ed25519 key
    Ed25519(MpcKeyTypeEd),
    #[strum_discriminants(strum(message = "secp256k1 - use Secp256K1 key derivation"))]
    /// Use Secp256K1 key
    Secp256k1(MpcKeyTypeSecp),
}

#[derive(Clone)]
pub struct MpcKeyTypeContext {
    global_context: crate::GlobalContext,
    controllable_account_id: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
    key_type: near_crypto::KeyType,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PublicKeyFromMpcContext)]
#[interactive_clap(output_context = MpcKeyTypeSecpContext)]
pub struct MpcKeyTypeSecp {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDerivationPath,
}

#[derive(Clone)]
struct MpcKeyTypeSecpContext(MpcKeyTypeContext);

impl MpcKeyTypeSecpContext {
    fn from_previous_context(
        previous_context: PublicKeyFromMpcContext,
        _scope: &<MpcKeyTypeSecp as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            global_context: previous_context.global_context,
            controllable_account_id: previous_context.controllable_account_id,
            admin_account_id: previous_context.admin_account_id,
            key_type: near_crypto::KeyType::SECP256K1,
        }))
    }
}

impl From<MpcKeyTypeSecpContext> for MpcKeyTypeContext {
    fn from(item: MpcKeyTypeSecpContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PublicKeyFromMpcContext)]
#[interactive_clap(output_context = MpcKeyTypeEdContext)]
pub struct MpcKeyTypeEd {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDerivationPath,
}

#[derive(Clone)]
struct MpcKeyTypeEdContext(MpcKeyTypeContext);

impl MpcKeyTypeEdContext {
    fn from_previous_context(
        previous_context: PublicKeyFromMpcContext,
        _scope: &<MpcKeyTypeEd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            global_context: previous_context.global_context,
            controllable_account_id: previous_context.controllable_account_id,
            admin_account_id: previous_context.admin_account_id,
            key_type: near_crypto::KeyType::ED25519,
        }))
    }
}

impl From<MpcKeyTypeEdContext> for MpcKeyTypeContext {
    fn from(item: MpcKeyTypeEdContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = MpcKeyTypeContext)]
#[interactive_clap(output_context = MpcDerivationPathContext)]
pub struct MpcDerivationPath {
    #[interactive_clap(skip_default_input_arg, always_quote)]
    /// What is the derivation path?
    derivation_path: String,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct MpcDerivationPathContext {
    global_context: crate::GlobalContext,
    admin_account_id: near_primitives::types::AccountId,
    key_type: near_crypto::KeyType,
    derivation_path: String,
}

impl MpcDerivationPathContext {
    fn from_previous_context(
        previous_context: MpcKeyTypeContext,
        scope: &<MpcDerivationPath as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            admin_account_id: previous_context.admin_account_id,
            key_type: previous_context.key_type,
            derivation_path: scope.derivation_path.clone(),
        })
    }
}

impl MpcDerivationPath {
    fn input_derivation_path(
        context: &MpcKeyTypeContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let derivation_path = inquire::Text::new("What is the derivation path?")
            .with_initial_value(&format!(
                "{}-{}",
                context.admin_account_id, context.controllable_account_id,
            ))
            .prompt()?;
        Ok(Some(derivation_path))
    }
}

impl From<MpcDerivationPathContext> for crate::network::NetworkContext {
    fn from(item: MpcDerivationPathContext) -> Self {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let derived_public_key =
                        crate::transaction_signature_options::sign_with_mpc::derive_public_key(
                            &network_config.get_mpc_contract_account_id()?,
                            &item.admin_account_id,
                            &item.derivation_path,
                            &item.key_type,
                            network_config,
                        )?;

                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        item.global_context.verbosity
                    {
                        eprint!("Public key (printed to stdout): ");
                    }
                    println!("{derived_public_key}");

                    Ok(())
                }
            });

        Self {
            config: item.global_context.config,
            // NOTE: We cannot determine MPC contract AccountId until we get NetworkConfig where
            // it's stored, resulting in passing only signer AccountId
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback,
        }
    }
}
