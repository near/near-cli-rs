use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = AddKeyWithMpcDerivedKeyContext)]
pub struct AddKeyWithMpcDerivedKey {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the Admin account address?
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subcommand)]
    /// What is key type for deriving key?
    mpc_key_type: MpcKeyType,
}

#[derive(Debug, Clone)]
pub struct AddKeyWithMpcDerivedKeyContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    key_permission: near_primitives::account::AccessKeyPermission,
    admin_account_id: near_primitives::types::AccountId,
}

impl AddKeyWithMpcDerivedKeyContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        scope: &<AddKeyWithMpcDerivedKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if previous_context.global_context.offline {
            eprintln!("\nInternet connection is required to derive key from MPC contract!");
            return Err(color_eyre::eyre::eyre!(
                "Internet connection is required to derive key from MPC contract!"
            ));
        }

        Ok(AddKeyWithMpcDerivedKeyContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            key_permission: previous_context.permission,
            admin_account_id: scope.admin_account_id.clone().into(),
        })
    }
}

impl AddKeyWithMpcDerivedKey {
    pub fn input_admin_account_id(
        context: &super::access_key_type::AccessTypeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the Admin account address?",
        )
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AddKeyWithMpcDerivedKeyContext)]
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
    signer_account_id: near_primitives::types::AccountId,
    key_permission: near_primitives::account::AccessKeyPermission,
    admin_account_id: near_primitives::types::AccountId,
    key_type: near_crypto::KeyType,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AddKeyWithMpcDerivedKeyContext)]
#[interactive_clap(output_context = MpcKeyTypeSecpContext)]
pub struct MpcKeyTypeSecp {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDeriveKeyToAdd,
}

#[derive(Clone)]
pub struct MpcKeyTypeSecpContext(MpcKeyTypeContext);

impl MpcKeyTypeSecpContext {
    fn from_previous_context(
        previous_context: AddKeyWithMpcDerivedKeyContext,
        _scope: &<MpcKeyTypeSecp as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            global_context: previous_context.global_context,
            admin_account_id: previous_context.admin_account_id,
            signer_account_id: previous_context.signer_account_id,
            key_permission: previous_context.key_permission,
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
#[interactive_clap(input_context = AddKeyWithMpcDerivedKeyContext)]
#[interactive_clap(output_context = MpcKeyTypeEdContext)]
pub struct MpcKeyTypeEd {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDeriveKeyToAdd,
}

#[derive(Clone)]
pub struct MpcKeyTypeEdContext(MpcKeyTypeContext);

impl MpcKeyTypeEdContext {
    fn from_previous_context(
        previous_context: AddKeyWithMpcDerivedKeyContext,
        _scope: &<MpcKeyTypeEd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            key_permission: previous_context.key_permission,
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
#[interactive_clap(output_context = MpcDeriveKeyToAddContext)]
pub struct MpcDeriveKeyToAdd {
    #[interactive_clap(skip_default_input_arg, always_quote)]
    /// What is the derivation path?
    derivation_path: String,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct MpcDeriveKeyToAddContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
    key_permission: near_primitives::account::AccessKeyPermission,
    key_type: near_crypto::KeyType,
    derivation_path: String,
}

impl MpcDeriveKeyToAdd {
    pub fn input_derivation_path(
        context: &MpcKeyTypeContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let derivation_path = inquire::Text::new("What is the derivation path?")
            .with_initial_value(&format!(
                "{}-{}",
                context.admin_account_id, context.signer_account_id
            ))
            .prompt()?;
        Ok(Some(derivation_path))
    }
}

impl MpcDeriveKeyToAddContext {
    pub fn from_previous_context(
        previous_context: MpcKeyTypeContext,
        scope: &<MpcDeriveKeyToAdd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            admin_account_id: previous_context.admin_account_id,
            key_permission: previous_context.key_permission,
            key_type: previous_context.key_type,
            derivation_path: scope.derivation_path.clone(),
        })
    }
}

impl From<MpcDeriveKeyToAddContext> for crate::commands::ActionContext {
    fn from(item: MpcDeriveKeyToAddContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback = std::sync::Arc::new({
            let signer_account_id = item.signer_account_id.clone();

            move |network_config| {
                let derived_public_key = crate::transaction_signature_options::sign_with_mpc::derive_public_key(
                    &network_config.get_mpc_contract_account_id()?,
                    &item.admin_account_id,
                    &item.derivation_path,
                    &item.key_type,
                    network_config,
                )?;

                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: signer_account_id.clone(),
                    receiver_id: signer_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::AddKey(Box::new(
                        near_primitives::transaction::AddKeyAction {
                            public_key: derived_public_key,
                            access_key: near_primitives::account::AccessKey {
                                nonce: 0,
                                permission: item.key_permission.clone(),
                            },
                        },
                    ))],
                })
            }
        });

        Self {
            global_context: item.global_context,
            // NOTE: We cannot determine MPC contract AccountId until we get NetworkConfig where
            // it's stored, resulting in passing only signer AccountId
            interacting_with_account_ids: vec![item.signer_account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
