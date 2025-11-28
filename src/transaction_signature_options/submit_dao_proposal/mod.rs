use std::str::FromStr;

mod dao_kind_arguments;
pub mod dao_sign_with;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = DaoProposalContext)]
pub struct DaoProposal {
    #[interactive_clap(skip_default_input_arg)]
    /// What is dao account ID?
    dao_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subargs)]
    /// Proposal function params
    proposal_arguments: DaoProposalArguments,
}

#[derive(Clone)]
pub struct DaoProposalContext {
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
    dao_account_id: near_primitives::types::AccountId,
    receiver_id: near_primitives::types::AccountId,
    proposal_kind: dao_kind_arguments::ProposalKind,
}

impl DaoProposalContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<DaoProposal as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let proposal_kind =
            dao_kind_arguments::ProposalKind::try_from(&previous_context.prepopulated_transaction)?;

        Ok(Self {
            global_context: previous_context.global_context,
            network_config: previous_context.network_config,
            dao_account_id: scope.dao_account_id.clone().into(),
            receiver_id: previous_context.prepopulated_transaction.signer_id.clone(),
            proposal_kind,
        })
    }
}

impl DaoProposal {
    pub fn input_dao_account_id(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the DAO member account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DaoProposalContext)]
#[interactive_clap(output_context = DaoProposalArgumentsContext)]
pub struct DaoProposalArguments {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter proposal description:
    proposal_description: String,
    #[interactive_clap(named_arg)]
    /// Enter gas amount for DAO proposal:
    prepaid_gas: PrepaidGas,
}

#[derive(Clone)]
pub struct DaoProposalArgumentsContext {
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
    dao_account_id: near_primitives::types::AccountId,
    receiver_id: near_primitives::types::AccountId,
    proposal_args: Vec<u8>,
}

impl DaoProposalArgumentsContext {
    pub fn from_previous_context(
        previous_context: DaoProposalContext,
        scope: &<DaoProposalArguments as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let proposal_args = serde_json::to_vec(&serde_json::json!({
            "proposal": {
                "description": scope.proposal_description,
                "kind": previous_context.proposal_kind,
            },
        }))?;

        Ok(Self {
            global_context: previous_context.global_context,
            network_config: previous_context.network_config,
            dao_account_id: previous_context.dao_account_id,
            receiver_id: previous_context.receiver_id,
            proposal_args,
        })
    }
}

impl DaoProposalArguments {
    pub fn input_proposal_description(
        _context: &DaoProposalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        match cliclack::input("Input proposal description:").interact() {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DaoProposalArgumentsContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas amount for DAO proposal:
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for DAO proposal:
    attached_deposit: Deposit,
}

#[derive(Clone)]
pub struct PrepaidGasContext {
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
    dao_account_id: near_primitives::types::AccountId,
    receiver_id: near_primitives::types::AccountId,
    proposal_args: Vec<u8>,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: DaoProposalArgumentsContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            network_config: previous_context.network_config,
            dao_account_id: previous_context.dao_account_id,
            receiver_id: previous_context.receiver_id,
            proposal_args: previous_context.proposal_args,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    pub fn input_gas(
        _context: &DaoProposalArgumentsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        match cliclack::input(
            "What is the gas limit for adding DAO proposal (if unsure, keep 10 Tgas)?",
        )
        .default_input("10 TeraGas")
        .validate(|s: &String| {
            let gas = near_gas::NearGas::from_str(s).map_err(|err| err.to_string())?;
            if gas > near_gas::NearGas::from_tgas(300) {
                Err("You need to enter a value of no more than 300 TeraGas".to_string())
            } else {
                Ok(())
            }
        })
        .interact()
        {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context=PrepaidGasContext)]
#[interactive_clap(output_context=DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for DAO proposal:
    deposit: crate::types::near_token::NearToken,
    #[interactive_clap(subcommand)]
    transaction_signature_options: dao_sign_with::DaoSignWith,
}

#[derive(Clone)]
pub struct DepositContext {
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
    dao_account_id: near_primitives::types::AccountId,
    receiver_id: near_primitives::types::AccountId,
    proposal_args: Vec<u8>,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            network_config: previous_context.network_config,
            dao_account_id: previous_context.dao_account_id,
            receiver_id: previous_context.receiver_id,
            proposal_args: previous_context.proposal_args,
            gas: previous_context.gas,
            deposit: scope.deposit,
        })
    }
}

impl Deposit {
    pub fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        match cliclack::input(
            "Enter deposit for adding DAO proposal (example: 10 NEAR or 0.5 near or 10000 yoctonear):",
        )
        .default_input("0 NEAR")
        .interact()
        {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

impl From<DepositContext> for crate::commands::TransactionContext {
    fn from(item: DepositContext) -> Self {
        let new_prepopulated_transaction = crate::commands::PrepopulatedTransaction {
            signer_id: item.dao_account_id,
            receiver_id: item.receiver_id,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                Box::new(near_primitives::transaction::FunctionCallAction {
                    method_name: "add_proposal".to_string(),
                    args: item.proposal_args.clone(),
                    gas: item.gas.as_gas(),
                    deposit: item.deposit.as_yoctonear(),
                }),
            )],
        };

        tracing::info!(
            "{}{}",
            "Unsigned DAO proposal",
            crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                &new_prepopulated_transaction,
            ))
        );

        Self {
            global_context: item.global_context,
            network_config: item.network_config,
            prepopulated_transaction: new_prepopulated_transaction,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
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
