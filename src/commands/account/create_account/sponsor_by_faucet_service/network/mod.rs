use color_eyre::eyre::{ContextCompat, WrapErr};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SponsorServiceContext)]
#[interactive_clap(output_context = NetworkContext)]
pub struct Network {
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    pub submit: Submit,
}

#[derive(Clone)]
pub struct NetworkContext {
    new_account_id: crate::types::account_id::AccountId,
    public_key: near_crypto::PublicKey,
    network_config: crate::config::NetworkConfig,
    on_after_getting_network_callback: OnAfterGettingNetworkCallback,
    on_before_creating_account_callback: OnBeforeCreatingAccountCallback,
}

impl NetworkContext {
    pub fn from_previous_context(
        previous_context: super::SponsorServiceContext,
        scope: &<Network as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let networks = previous_context.config.network_connection.clone();
        let network_config = networks
            .get(&scope.network_name)
            .wrap_err(sysexits::ExitCode::DataErr)
            .wrap_err("Failed to get network config!")?
            .clone();

        let mut info_str: String = String::new();
        info_str.push_str(&format!(
            "\n{:<13} {}",
            "signer_id:", &network_config.network_name
        ));
        info_str.push_str("\nactions:");
        info_str.push_str(&format!(
            "\n{:>5} {:<20} {}",
            "--", "create account:", &previous_context.new_account_id
        ));
        info_str.push_str(&format!("\n{:>5} {:<20}", "--", "add access key:"));
        info_str.push_str(&format!(
            "\n{:>18} {:<13} {}",
            "", "public key:", &previous_context.public_key
        ));
        info_str.push_str(&format!("\n{:>18} {:<13} FullAccess", "", "permission:"));
        info_str.push('\n');

        tracing::info!(
            "Your transaction:{}",
            crate::common::indent_payload(&info_str)
        );

        Ok(Self {
            new_account_id: previous_context.new_account_id,
            public_key: previous_context.public_key,
            network_config,
            on_after_getting_network_callback: previous_context.on_after_getting_network_callback,
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        })
    }
}

impl Network {
    fn input_network_name(
        context: &super::SponsorServiceContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[context.new_account_id.clone().into()])
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NetworkContext)]
#[interactive_clap(output_context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to proceed?
pub enum Submit {
    #[strum_discriminants(strum(message = "create      - Create a new account"))]
    Create,
}

#[derive(Debug, Clone)]
pub struct SubmitContext;

impl SubmitContext {
    #[tracing::instrument(name = "Creating a new account ...", skip_all)]
    pub fn from_previous_context(
        previous_context: NetworkContext,
        _scope: &<Submit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let storage_message =
            (previous_context.on_after_getting_network_callback)(&previous_context.network_config)?;
        (previous_context.on_before_creating_account_callback)(
            &previous_context.network_config,
            &previous_context.new_account_id,
            &previous_context.public_key,
            storage_message,
        )?;
        Ok(Self)
    }
}

pub type OnAfterGettingNetworkCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig) -> color_eyre::eyre::Result<String>>;

pub type OnBeforeCreatingAccountCallback = std::sync::Arc<
    dyn Fn(
        &crate::config::NetworkConfig,
        &crate::types::account_id::AccountId,
        &near_crypto::PublicKey,
        String,
    ) -> crate::CliResult,
>;
