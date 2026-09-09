macro_rules! impl_add_action_level {
    ($level:ident, $next_level:ident) => {
        #[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
        pub mod $level {
            use strum::{EnumDiscriminants, EnumIter, EnumMessage};

            #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
            #[interactive_clap(context = super::ConstructTransactionContext)]
            #[strum_discriminants(derive(EnumMessage, EnumIter))]
            /// Select an action that you want to add to the action:
            pub enum NextAction {
                #[strum_discriminants(strum(message = "add-action   - Select a new action"))]
                /// Choose next action
                AddAction(self::add_action::AddAction),
                #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
                /// Go to transaction signing
                Skip(super::skip_action::SkipAction),
            }

            pub mod add_action {
                use strum::{EnumDiscriminants, EnumIter, EnumMessage};

                #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                #[interactive_clap(context = super::super::ConstructTransactionContext)]
                pub struct AddAction {
                    #[interactive_clap(subcommand)]
                    pub action: ActionSubcommand,
                }

                #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
                #[interactive_clap(context = super::super::ConstructTransactionContext)]
                #[strum_discriminants(derive(EnumMessage, EnumIter))]
                /// Select an action that you want to add to the action:
                pub enum ActionSubcommand {
                    #[strum_discriminants(strum(
                        message = "transfer               - The transfer is carried out in NEAR tokens"
                    ))]
                    /// Specify data for transfer tokens
                    Transfer(self::transfer::TransferAction),
                    #[strum_discriminants(strum(
                        message = "function-call          - Execute function (contract method)"
                    ))]
                    /// Specify data to call the function
                    FunctionCall(self::call_function::FunctionCallAction),
                    #[strum_discriminants(strum(message = "stake                  - Stake NEAR Tokens"))]
                    /// Specify data to stake NEAR Tokens
                    Stake(self::stake::StakeAction),
                    #[strum_discriminants(strum(message = "create-account         - Create a new sub-account"))]
                    /// Specify data to create a sub-account
                    CreateAccount(self::create_account::CreateAccountAction),
                    #[strum_discriminants(strum(message = "delete-account         - Delete an account"))]
                    /// Specify data to delete an account
                    DeleteAccount(self::delete_account::DeleteAccountAction),
                    #[strum_discriminants(strum(
                        message = "add-key                - Add an access key to an account"
                    ))]
                    /// Specify the data to add an access key to the account
                    AddKey(self::add_key::AddKeyAction),
                    #[strum_discriminants(strum(
                        message = "delete-key             - Delete an access key from an account"
                    ))]
                    /// Specify the data to delete the access key to the account
                    DeleteKey(self::delete_key::DeleteKeyAction),
                    #[strum_discriminants(strum(message = "deploy                 - Add a new contract code"))]
                    /// Specify the details to deploy the contract code
                    DeployContract(self::deploy_contract::DeployContractAction),
                    #[strum_discriminants(strum(
                        message = "deploy-global-contract - Add a new global contract code"
                    ))]
                    /// Specify the details to deploy the global contract code
                    DeployGlobalContract(self::deploy_global_contract::DeployGlobalContractAction),
                    #[strum_discriminants(strum(
                        message = "use-global-contract    - Use a global contract to re-use the pre-deployed on-chain code"
                    ))]
                    /// Specify the details to use the global contract
                    UseGlobalContract(self::use_global_contract::UseGlobalContractAction),
                }

                pub mod add_key {
                    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = super::super::super::ConstructTransactionContext)]
                    pub struct AddKeyAction {
                        #[interactive_clap(subcommand)]
                        permission: AccessKeyPermission,
                    }

                    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = super::super::super::ConstructTransactionContext)]
                    #[strum_discriminants(derive(EnumMessage, EnumIter))]
                    /// Select a permission that you want to add to the access key:
                    pub enum AccessKeyPermission {
                        #[strum_discriminants(strum(
                            message = "grant-full-access           - A permission with full access"
                        ))]
                        /// Provide data for a full access key
                        GrantFullAccess(self::access_key_type::FullAccessType),
                        #[strum_discriminants(strum(
                            message = "grant-function-call-access  - A permission with function call"
                        ))]
                        /// Provide data for a function-call access key
                        GrantFunctionCallAccess(self::access_key_type::FunctionCallType),
                    }

                    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = self::access_key_type::AccessKeyPermissionContext)]
                    #[strum_discriminants(derive(EnumMessage, EnumIter))]
                    /// Add an access key for this account:
                    pub enum AccessKeyMode {
                        #[strum_discriminants(strum(
                            message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
                        ))]
                        /// Use the provided seed phrase manually
                        UseManuallyProvidedSeedPhrase(
                            self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
                        ),
                        #[strum_discriminants(strum(
                            message = "use-manually-provided-public-key  - Use the provided public key manually"
                        ))]
                        /// Use the provided public key manually
                        UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
                    }

                    pub mod access_key_type {
                        use std::str::FromStr;

                        use inquire::{CustomType, Select, Text};

                        #[derive(Debug, Clone)]
                        pub struct AccessKeyPermissionContext {
                            pub global_context: crate::GlobalContext,
                            pub signer_account_id: near_primitives::types::AccountId,
                            pub receiver_account_id: near_primitives::types::AccountId,
                            pub actions: Vec<near_primitives::transaction::Action>,
                            pub access_key_permission: near_primitives::account::AccessKeyPermission,
                            pub sign_as_delegate_action: bool,
                        }

                        #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                        #[interactive_clap(input_context = super::super::super::super::ConstructTransactionContext)]
                        #[interactive_clap(output_context = FullAccessTypeContext)]
                        pub struct FullAccessType {
                            #[interactive_clap(subcommand)]
                            access_key_mode: super::AccessKeyMode,
                        }

                        #[derive(Debug, Clone)]
                        pub struct FullAccessTypeContext(AccessKeyPermissionContext);

                        impl FullAccessTypeContext {
                            pub fn from_previous_context(
                                previous_context: super::super::super::super::ConstructTransactionContext,
                                _scope: &<FullAccessType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                            ) -> color_eyre::eyre::Result<Self> {
                                Ok(Self(AccessKeyPermissionContext {
                                    global_context: previous_context.global_context,
                                    signer_account_id: previous_context.signer_account_id,
                                    receiver_account_id: previous_context.receiver_account_id,
                                    actions: previous_context.actions,
                                    access_key_permission: near_primitives::account::AccessKeyPermission::FullAccess,
                                    sign_as_delegate_action: previous_context.sign_as_delegate_action,
                                }))
                            }
                        }

                        impl From<FullAccessTypeContext> for AccessKeyPermissionContext {
                            fn from(item: FullAccessTypeContext) -> Self {
                                item.0
                            }
                        }

                        #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                        #[interactive_clap(input_context = super::super::super::super::ConstructTransactionContext)]
                        #[interactive_clap(output_context = FunctionCallTypeContext)]
                        pub struct FunctionCallType {
                            #[interactive_clap(long)]
                            #[interactive_clap(skip_default_input_arg)]
                            allowance: crate::types::near_allowance::NearAllowance,
                            #[interactive_clap(long)]
                            /// Enter the contract account ID that this access key can be used to sign call function transactions for:
                            contract_account_id: crate::types::account_id::AccountId,
                            #[interactive_clap(long)]
                            #[interactive_clap(skip_default_input_arg)]
                            function_names: crate::types::vec_string::VecString,
                            #[interactive_clap(subcommand)]
                            access_key_mode: super::AccessKeyMode,
                        }

                        #[derive(Debug, Clone)]
                        pub struct FunctionCallTypeContext(AccessKeyPermissionContext);

                        impl FunctionCallTypeContext {
                            pub fn from_previous_context(
                                previous_context: super::super::super::super::ConstructTransactionContext,
                                scope: &<FunctionCallType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                            ) -> color_eyre::eyre::Result<Self> {
                                let access_key_permission = near_primitives::account::AccessKeyPermission::FunctionCall(
                                    near_primitives::account::FunctionCallPermission {
                                        allowance: scope.allowance.optional_near_token().map(Into::into),
                                        receiver_id: scope.contract_account_id.to_string(),
                                        method_names: scope.function_names.clone().into(),
                                    },
                                );
                                Ok(Self(AccessKeyPermissionContext {
                                    global_context: previous_context.global_context,
                                    signer_account_id: previous_context.signer_account_id,
                                    receiver_account_id: previous_context.receiver_account_id,
                                    actions: previous_context.actions,
                                    access_key_permission,
                                    sign_as_delegate_action: previous_context.sign_as_delegate_action,
                                }))
                            }
                        }

                        impl From<FunctionCallTypeContext> for AccessKeyPermissionContext {
                            fn from(item: FunctionCallTypeContext) -> Self {
                                item.0
                            }
                        }

                        impl FunctionCallType {
                            pub fn input_function_names(
                                _context: &super::super::super::super::ConstructTransactionContext,
                            ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
                                #[derive(strum_macros::Display)]
                                enum ConfirmOptions {
                                    #[strum(
                                        to_string = "Yes, I want to input a list of function names that can be called when transaction is signed by this access key"
                                    )]
                                    Yes,
                                    #[strum(to_string = "No, I allow it to call any functions on the specified contract")]
                                    No,
                                }
                                let select_choose_input = Select::new(
                                    "Would you like the access key to be valid exclusively for calling specific functions on the contract?",
                                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                                )
                                .prompt()?;
                                if let ConfirmOptions::Yes = select_choose_input {
                                    let mut input_function_names =
                                            Text::new("Enter a comma-separated list of function names that will be allowed to be called in a transaction signed by this access key:")
                                                .prompt()?;
                                    if input_function_names.contains('\"') {
                                        input_function_names.clear()
                                    };
                                    if input_function_names.is_empty() {
                                        Ok(Some(crate::types::vec_string::VecString(vec![])))
                                    } else {
                                        Ok(Some(crate::types::vec_string::VecString::from_str(
                                            &input_function_names,
                                        )?))
                                    }
                                } else {
                                    Ok(Some(crate::types::vec_string::VecString(vec![])))
                                }
                            }

                            pub fn input_allowance(
                                _context: &super::super::super::super::ConstructTransactionContext,
                            ) -> color_eyre::eyre::Result<Option<crate::types::near_allowance::NearAllowance>> {
                                let allowance_near_balance: crate::types::near_allowance::NearAllowance =
                                    CustomType::new("Enter the allowance, a budget this access key can use to pay for transaction fees (example: 10 NEAR or 0.5 NEAR or 10000 yoctonear):")
                                        .with_starting_input("unlimited")
                                        .prompt()?;
                                Ok(Some(allowance_near_balance))
                            }
                        }
                    }

                    pub mod use_manually_provided_seed_phrase {
                        use std::str::FromStr;

                        #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                        #[interactive_clap(input_context = super::access_key_type::AccessKeyPermissionContext)]
                        #[interactive_clap(output_context = AddAccessWithSeedPhraseActionContext)]
                        pub struct AddAccessWithSeedPhraseAction {
                            /// Enter the seed_phrase:
                            master_seed_phrase: String,
                            #[interactive_clap(subcommand)]
                            next_action: super::super::super::super::$next_level::NextAction,
                        }

                        #[derive(Debug, Clone)]
                        pub struct AddAccessWithSeedPhraseActionContext(
                            super::super::super::super::ConstructTransactionContext,
                        );

                        impl AddAccessWithSeedPhraseActionContext {
                            pub fn from_previous_context(
                                previous_context: super::access_key_type::AccessKeyPermissionContext,
                                scope: &<AddAccessWithSeedPhraseAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                            ) -> color_eyre::eyre::Result<Self> {
                                let seed_phrase_hd_path_default =
                                    near_slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
                                let public_key = crate::common::get_public_key_from_seed_phrase(
                                    seed_phrase_hd_path_default,
                                    &scope.master_seed_phrase,
                                )?;
                                let access_key = near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: previous_context.access_key_permission,
                                };
                                let action = near_primitives::transaction::Action::AddKey(Box::new(
                                    near_primitives::transaction::AddKeyAction {
                                        public_key,
                                        access_key,
                                    },
                                ));
                                let mut actions = previous_context.actions;
                                actions.push(action);
                                Ok(Self(
                                    super::super::super::super::ConstructTransactionContext {
                                        global_context: previous_context.global_context,
                                        signer_account_id: previous_context.signer_account_id,
                                        receiver_account_id: previous_context.receiver_account_id,
                                        actions,
                                        sign_as_delegate_action: previous_context.sign_as_delegate_action,
                                    },
                                ))
                            }
                        }

                        impl From<AddAccessWithSeedPhraseActionContext>
                            for super::super::super::super::ConstructTransactionContext
                        {
                            fn from(item: AddAccessWithSeedPhraseActionContext) -> Self {
                                item.0
                            }
                        }
                    }

                    pub mod use_public_key {
                        #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                        #[interactive_clap(input_context = super::access_key_type::AccessKeyPermissionContext)]
                        #[interactive_clap(output_context = AddAccessKeyActionContext)]
                        pub struct AddAccessKeyAction {
                            /// Enter the public key:
                            public_key: crate::types::public_key::PublicKey,
                            #[interactive_clap(subcommand)]
                            next_action: super::super::super::super::$next_level::NextAction,
                        }

                        #[derive(Debug, Clone)]
                        pub struct AddAccessKeyActionContext(super::super::super::super::ConstructTransactionContext);

                        impl AddAccessKeyActionContext {
                            pub fn from_previous_context(
                                previous_context: super::access_key_type::AccessKeyPermissionContext,
                                scope: &<AddAccessKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                            ) -> color_eyre::eyre::Result<Self> {
                                let access_key = near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: previous_context.access_key_permission,
                                };
                                let action = near_primitives::transaction::Action::AddKey(Box::new(
                                    near_primitives::transaction::AddKeyAction {
                                        public_key: scope.public_key.clone().into(),
                                        access_key,
                                    },
                                ));
                                let mut actions = previous_context.actions;
                                actions.push(action);
                                Ok(Self(
                                    super::super::super::super::ConstructTransactionContext {
                                        global_context: previous_context.global_context,
                                        signer_account_id: previous_context.signer_account_id,
                                        receiver_account_id: previous_context.receiver_account_id,
                                        actions,
                                        sign_as_delegate_action: previous_context.sign_as_delegate_action,
                                    },
                                ))
                            }
                        }

                        impl From<AddAccessKeyActionContext> for super::super::super::super::ConstructTransactionContext {
                            fn from(item: AddAccessKeyActionContext) -> Self {
                                item.0
                            }
                        }
                    }
                }

                pub mod call_function {
                    use inquire::CustomType;

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = FunctionCallActionContext)]
                    pub struct FunctionCallAction {
                        #[interactive_clap(skip_default_input_arg)]
                        /// What is the name of the function?
                        function_name: String,
                        #[interactive_clap(value_enum)]
                        #[interactive_clap(skip_default_input_arg)]
                        /// How do you want to pass the function call arguments?
                        function_args_type:
                            crate::commands::contract::call_function::call_function_args_type::FunctionArgsType,
                        /// Enter the arguments to this function:
                        function_args: String,
                        #[interactive_clap(named_arg)]
                        /// Enter gas for function call
                        prepaid_gas: PrepaidGas,
                    }

                    #[derive(Debug, Clone)]
                    pub struct FunctionCallActionContext {
                        global_context: crate::GlobalContext,
                        signer_account_id: near_primitives::types::AccountId,
                        receiver_account_id: near_primitives::types::AccountId,
                        actions: Vec<near_primitives::transaction::Action>,
                        function_name: String,
                        function_args: Vec<u8>,
                        sign_as_delegate_action: bool,
                    }

                    impl FunctionCallActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<FunctionCallAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let function_args =
                                crate::commands::contract::call_function::call_function_args_type::function_args(
                                    scope.function_args.clone(),
                                    scope.function_args_type.clone(),
                                )?;
                            Ok(Self {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions: previous_context.actions,
                                function_name: scope.function_name.clone(),
                                function_args,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            })
                        }
                    }

                    impl FunctionCallAction {
                        fn input_function_args_type(
                            _context: &super::super::super::ConstructTransactionContext,
                        ) -> color_eyre::eyre::Result<
                            Option<crate::commands::contract::call_function::call_function_args_type::FunctionArgsType>,
                        > {
                            crate::commands::contract::call_function::call_function_args_type::input_function_args_type(
                            )
                        }

                        fn input_function_name(
                            context: &super::super::super::ConstructTransactionContext,
                        ) -> color_eyre::eyre::Result<Option<String>> {
                            crate::commands::contract::call_function::input_call_function_name(
                                &context.global_context,
                                &context.receiver_account_id,
                            )
                        }
                    }

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = FunctionCallActionContext)]
                    #[interactive_clap(output_context = PrepaidGasContext)]
                    pub struct PrepaidGas {
                        #[interactive_clap(skip_default_input_arg)]
                        /// Enter gas for function call:
                        gas: crate::common::NearGas,
                        #[interactive_clap(named_arg)]
                        /// Enter deposit for a function call
                        attached_deposit: Deposit,
                    }

                    #[derive(Debug, Clone)]
                    pub struct PrepaidGasContext {
                        global_context: crate::GlobalContext,
                        signer_account_id: near_primitives::types::AccountId,
                        receiver_account_id: near_primitives::types::AccountId,
                        actions: Vec<near_primitives::transaction::Action>,
                        function_name: String,
                        function_args: Vec<u8>,
                        gas: crate::common::NearGas,
                        sign_as_delegate_action: bool,
                    }

                    impl PrepaidGasContext {
                        pub fn from_previous_context(
                            previous_context: FunctionCallActionContext,
                            scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            Ok(Self {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions: previous_context.actions,
                                function_name: previous_context.function_name,
                                function_args: previous_context.function_args,
                                gas: scope.gas,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            })
                        }
                    }

                    impl PrepaidGas {
                        fn input_gas(
                            _context: &FunctionCallActionContext,
                        ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
                            Ok(Some(
                                CustomType::new("Enter gas for function call:")
                                    .with_starting_input("100 TeraGas")
                                    .with_validator(move |gas: &crate::common::NearGas| {
                                        if gas > &near_gas::NearGas::from_tgas(1000) {
                                            Ok(inquire::validator::Validation::Invalid(
                                                inquire::validator::ErrorMessage::Custom(
                                                    "You need to enter a value of no more than 1000 TeraGas"
                                                        .to_string(),
                                                ),
                                            ))
                                        } else {
                                            Ok(inquire::validator::Validation::Valid)
                                        }
                                    })
                                    .prompt()?,
                            ))
                        }
                    }

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = PrepaidGasContext)]
                    #[interactive_clap(output_context = DepositContext)]
                    pub struct Deposit {
                        #[interactive_clap(skip_default_input_arg)]
                        /// Enter deposit for a function call:
                        deposit: crate::types::near_token::NearToken,
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct DepositContext(super::super::super::ConstructTransactionContext);

                    impl DepositContext {
                        pub fn from_previous_context(
                            previous_context: PrepaidGasContext,
                            scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::FunctionCall(Box::new(
                                near_primitives::transaction::FunctionCallAction {
                                    method_name: previous_context.function_name,
                                    args: previous_context.function_args,
                                    gas: near_primitives::gas::Gas::from_gas(previous_context.gas.as_gas()),
                                    deposit: scope.deposit.into(),
                                },
                            ));
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<DepositContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: DepositContext) -> Self {
                            item.0
                        }
                    }

                    impl Deposit {
                        fn input_deposit(
                            _context: &PrepaidGasContext,
                        ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
                            Ok(Some(
                                CustomType::new("Enter deposit for a function call (example: 10 NEAR or 0.5 near or 10000 yoctonear):")
                                    .with_starting_input("0 NEAR")
                                    .prompt()?
                            ))
                        }
                    }
                }

                pub mod create_account {
                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = CreateAccountActionContext)]
                    pub struct CreateAccountAction {
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct CreateAccountActionContext(super::super::super::ConstructTransactionContext);

                    impl CreateAccountActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            _scope: &<CreateAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::CreateAccount(
                                near_primitives::transaction::CreateAccountAction {},
                            );
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<CreateAccountActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: CreateAccountActionContext) -> Self {
                            item.0
                        }
                    }
                }

                pub mod delete_account {
                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = DeleteAccountActionContext)]
                    pub struct DeleteAccountAction {
                        #[interactive_clap(long)]
                        #[interactive_clap(skip_default_input_arg)]
                        /// Enter the beneficiary ID to delete this account ID:
                        beneficiary_id: crate::types::account_id::AccountId,
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct DeleteAccountActionContext(super::super::super::ConstructTransactionContext);

                    impl DeleteAccountActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<DeleteAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let beneficiary_id: near_primitives::types::AccountId = scope.beneficiary_id.clone().into();
                            if previous_context.signer_account_id == beneficiary_id {
                                return Err(color_eyre::eyre::eyre!(
                                    "Invalid beneficiary account ID.\nThe beneficiary account ID cannot be the same as the account ID being deleted."
                                ));
                            }
                            let action = near_primitives::transaction::Action::DeleteAccount(
                                near_primitives::transaction::DeleteAccountAction { beneficiary_id },
                            );
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<DeleteAccountActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: DeleteAccountActionContext) -> Self {
                            item.0
                        }
                    }

                    impl DeleteAccountAction {
                        pub fn input_beneficiary_id(
                            context: &super::super::super::ConstructTransactionContext,
                        ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
                            crate::commands::account::delete_account::BeneficiaryAccount::input_beneficiary_account_id(
                                &context.clone().into(),
                            )
                        }
                    }
                }

                pub mod delete_key {
                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = DeleteKeyActionContext)]
                    pub struct DeleteKeyAction {
                        #[interactive_clap(skip_default_input_arg)]
                        /// Enter the public keys you wish to delete (separated by comma):
                        public_keys: crate::types::public_key_list::PublicKeyList,
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct DeleteKeyActionContext(super::super::super::ConstructTransactionContext);

                    impl DeleteKeyActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<DeleteKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let public_keys: Vec<near_crypto::PublicKey> = scope.public_keys.clone().into();
                            let action: Vec<near_primitives::transaction::Action> = public_keys
                                .into_iter()
                                .map(|public_key| {
                                    near_primitives::transaction::Action::DeleteKey(Box::new(
                                        near_primitives::transaction::DeleteKeyAction { public_key },
                                    ))
                                })
                                .collect();
                            let mut actions = previous_context.actions;
                            actions.extend(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<DeleteKeyActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: DeleteKeyActionContext) -> Self {
                            item.0
                        }
                    }

                    impl DeleteKeyAction {
                        pub fn input_public_keys(
                            context: &super::super::super::ConstructTransactionContext,
                        ) -> color_eyre::eyre::Result<Option<crate::types::public_key_list::PublicKeyList>> {
                            crate::commands::account::delete_key::public_keys_to_delete::PublicKeyList::input_public_keys(
                                &context.clone().into(),
                            )
                        }
                    }
                }

                pub mod deploy_contract {
                    use color_eyre::eyre::Context;

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = super::super::super::ConstructTransactionContext)]
                    pub struct DeployContractAction {
                        #[interactive_clap(named_arg)]
                        /// Specify a path to wasm file
                        use_file: ContractFile,
                    }

                    #[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = ContractFileContext)]
                    pub struct ContractFile {
                        /// What is the file location of the contract?
                        pub file_path: crate::types::path_buf::PathBuf,
                        #[interactive_clap(subcommand)]
                        initialize: self::initialize_mode::InitializeMode,
                    }

                    #[derive(Debug, Clone)]
                    pub struct ContractFileContext(super::super::super::ConstructTransactionContext);

                    impl ContractFileContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
                                format!("Failed to open or read the file: {:?}.", scope.file_path.0,)
                            })?;
                            let action = near_primitives::transaction::Action::DeployContract(
                                near_primitives::transaction::DeployContractAction { code },
                            );
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<ContractFileContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: ContractFileContext) -> Self {
                            item.0
                        }
                    }

                    pub mod initialize_mode {
                        use strum::{EnumDiscriminants, EnumIter, EnumMessage};

                        #[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
                        #[interactive_clap(context = super::super::super::super::ConstructTransactionContext)]
                        #[strum_discriminants(derive(EnumMessage, EnumIter))]
                        /// Select the need for initialization:
                        pub enum InitializeMode {
                            /// Add an initialize
                            #[strum_discriminants(strum(message = "with-init-call     - Add an initialize"))]
                            WithInitCall(super::super::call_function::FunctionCallAction),
                            /// Don't add an initialize
                            #[strum_discriminants(strum(message = "without-init-call  - Don't add an initialize"))]
                            WithoutInitCall(NoInitialize),
                        }

                        #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                        #[interactive_clap(context = super::super::super::super::ConstructTransactionContext)]
                        pub struct NoInitialize {
                            #[interactive_clap(subcommand)]
                            next_action: super::super::super::super::$next_level::NextAction,
                        }
                    }
                }

                pub mod deploy_global_contract {
                    use color_eyre::eyre::Context;
                    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = DeployGlobalContractActionContext)]
                    pub struct DeployGlobalContractAction {
                        /// What is the file location of the contract?
                        pub file_path: crate::types::path_buf::PathBuf,
                        #[interactive_clap(subcommand)]
                        mode: DeployGlobalMode,
                    }

                    #[derive(Debug, Clone)]
                    pub struct DeployGlobalContractActionContext {
                        pub context: super::super::super::ConstructTransactionContext,
                        pub code: Vec<u8>,
                    }

                    impl DeployGlobalContractActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<DeployGlobalContractAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
                                format!("Failed to open or read the file: {:?}.", scope.file_path.0,)
                            })?;
                            Ok(Self {
                                context: previous_context,
                                code,
                            })
                        }
                    }

                    #[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = DeployGlobalContractActionContext)]
                    #[interactive_clap(output_context = DeployGlobalModeContext)]
                    #[strum_discriminants(derive(EnumMessage, EnumIter))]
                    #[non_exhaustive]
                    /// Choose a global contract deploy mode:
                    pub enum DeployGlobalMode {
                        #[strum_discriminants(strum(
                            message = "as-global-hash       - Deploy code as a global contract code hash (immutable)"
                        ))]
                        /// Deploy code as a global contract code hash (immutable)
                        AsGlobalHash(NextCommand),
                        #[strum_discriminants(strum(
                            message = "as-global-account-id - Deploy code as a global contract account ID (mutable)"
                        ))]
                        /// Deploy code as a global contract account ID (mutable)
                        AsGlobalAccountId(NextCommand),
                    }

                    #[derive(Debug, Clone)]
                    pub struct DeployGlobalModeContext(super::super::super::ConstructTransactionContext);

                    impl DeployGlobalModeContext {
                        pub fn from_previous_context(
                            previous_context: DeployGlobalContractActionContext,
                            scope: &<DeployGlobalMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::DeployGlobalContract(
                                near_primitives::action::DeployGlobalContractAction {
                                    code: previous_context.code.into(),
                                    deploy_mode: match scope {
                                        DeployGlobalModeDiscriminants::AsGlobalHash => {
                                            near_primitives::action::GlobalContractDeployMode::CodeHash
                                        }
                                        DeployGlobalModeDiscriminants::AsGlobalAccountId => {
                                            near_primitives::action::GlobalContractDeployMode::AccountId
                                        }
                                    },
                                },
                            );
                            let mut actions = previous_context.context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.context.global_context,
                                signer_account_id: previous_context.context.signer_account_id,
                                receiver_account_id: previous_context.context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<DeployGlobalModeContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: DeployGlobalModeContext) -> Self {
                            item.0
                        }
                    }

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = DeployGlobalModeContext)]
                    pub struct NextCommand {
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }
                }

                pub mod stake {
                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = StakeActionContext)]
                    pub struct StakeAction {
                        /// Enter the amount to stake: (example: 10000NEAR)
                        stake_amount: crate::types::near_token::NearToken,
                        /// Enter the public key of the validator key pair used on your NEAR node (see validator_key.json):
                        public_key: crate::types::public_key::PublicKey,
                        #[interactive_clap(subcommand)]
                        next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct StakeActionContext(super::super::super::ConstructTransactionContext);

                    impl StakeActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<StakeAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::Stake(Box::new(
                                near_primitives::transaction::StakeAction {
                                    stake: scope.stake_amount.into(),
                                    public_key: scope.public_key.clone().into(),
                                },
                            ));
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<StakeActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: StakeActionContext) -> Self {
                            item.0
                        }
                    }
                }

                pub mod transfer {
                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = TransferActionContext)]
                    pub struct TransferAction {
                        /// How many NEAR Tokens do you want to transfer? (example: 10 NEAR or 0.5 NEAR or 10000 yoctonear)
                        pub amount_in_near: crate::types::near_token::NearToken,
                        #[interactive_clap(subcommand)]
                        pub next_action: super::super::super::$next_level::NextAction,
                    }

                    #[derive(Debug, Clone)]
                    pub struct TransferActionContext(super::super::super::ConstructTransactionContext);

                    impl TransferActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<TransferAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::Transfer(
                                near_primitives::transaction::TransferAction {
                                    deposit: scope.amount_in_near.into(),
                                },
                            );
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<TransferActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: TransferActionContext) -> Self {
                            item.0
                        }
                    }
                }

                pub mod use_global_contract {
                    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = super::super::super::ConstructTransactionContext)]
                    pub struct UseGlobalContractAction {
                        #[interactive_clap(subcommand)]
                        mode: UseGlobalActionMode,
                    }

                    #[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(context = super::super::super::ConstructTransactionContext)]
                    #[strum_discriminants(derive(EnumMessage, EnumIter))]
                    #[non_exhaustive]
                    /// Choose a global contract deploy mode:
                    pub enum UseGlobalActionMode {
                        #[strum_discriminants(strum(
                            message = "use-global-hash       - Use a global contract code hash pre-deployed on-chain (immutable)"
                        ))]
                        /// Use a global contract code hash (immutable)
                        UseGlobalHash(UseHashAction),
                        #[strum_discriminants(strum(
                            message = "use-global-account-id - Use a global contract account ID pre-deployed on-chain (mutable)"
                        ))]
                        /// Use a global contract account ID (mutable)
                        UseGlobalAccountId(UseAccountIdAction),
                    }

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = UseHashActionContext)]
                    pub struct UseHashAction {
                        /// What is the hash of the global contract?
                        pub hash: crate::types::crypto_hash::CryptoHash,
                        #[interactive_clap(subcommand)]
                        initialize: super::deploy_contract::initialize_mode::InitializeMode,
                    }

                    #[derive(Debug, Clone)]
                    pub struct UseHashActionContext(super::super::super::ConstructTransactionContext);

                    impl UseHashActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<UseHashAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
                                near_primitives::action::UseGlobalContractAction {
                                    contract_identifier: near_primitives::action::GlobalContractIdentifier::CodeHash(
                                        scope.hash.into(),
                                    ),
                                },
                            ));
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }
                    impl From<UseHashActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: UseHashActionContext) -> Self {
                            item.0
                        }
                    }

                    #[derive(Debug, Clone, interactive_clap::InteractiveClap)]
                    #[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
                    #[interactive_clap(output_context = UseAccountIdActionContext)]
                    pub struct UseAccountIdAction {
                        #[interactive_clap(skip_default_input_arg)]
                        /// What is the account ID of the global contract?
                        pub account_id: crate::types::account_id::AccountId,
                        #[interactive_clap(subcommand)]
                        initialize: super::deploy_contract::initialize_mode::InitializeMode,
                    }

                    impl UseAccountIdAction {
                        pub fn input_account_id(
                            context: &super::super::super::ConstructTransactionContext,
                        ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
                            crate::common::input_non_signer_account_id_from_used_account_list(
                                &context.global_context.config.credentials_home_dir,
                                "What is the account ID of the global contract?",
                            )
                        }
                    }

                    #[derive(Debug, Clone)]
                    pub struct UseAccountIdActionContext(super::super::super::ConstructTransactionContext);

                    impl UseAccountIdActionContext {
                        pub fn from_previous_context(
                            previous_context: super::super::super::ConstructTransactionContext,
                            scope: &<UseAccountIdAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
                        ) -> color_eyre::eyre::Result<Self> {
                            let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
                                near_primitives::action::UseGlobalContractAction {
                                    contract_identifier: near_primitives::action::GlobalContractIdentifier::AccountId(
                                        scope.account_id.clone().into(),
                                    ),
                                },
                            ));
                            let mut actions = previous_context.actions;
                            actions.push(action);
                            Ok(Self(super::super::super::ConstructTransactionContext {
                                global_context: previous_context.global_context,
                                signer_account_id: previous_context.signer_account_id,
                                receiver_account_id: previous_context.receiver_account_id,
                                actions,
                                sign_as_delegate_action: previous_context.sign_as_delegate_action,
                            }))
                        }
                    }

                    impl From<UseAccountIdActionContext> for super::super::super::ConstructTransactionContext {
                        fn from(item: UseAccountIdActionContext) -> Self {
                            item.0
                        }
                    }
                }
            }
        }
    };
}

pub(crate) use impl_add_action_level;
