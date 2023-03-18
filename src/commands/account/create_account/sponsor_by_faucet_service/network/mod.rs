use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SponsorServiceContext)]
#[interactive_clap(output_context = NetworkContext)]
pub struct Network {
    /// What is the name of the network
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
        let networks = previous_context.config.networks.clone();
        let network_config = networks
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();

        println!("\nYour transaction:");
        println!("{:<13} {}", "signer_id:", &network_config.network_name);
        println!("actions:");
        println!(
            "{:>5} {:<20} {}",
            "--", "create account:", &previous_context.new_account_id
        );
        println!("{:>5} {:<20}", "--", "add access key:");
        println!(
            "{:>18} {:<13} {}",
            "", "public key:", &previous_context.public_key
        );
        println!("{:>18} {:<13} FullAccess", "", "permission:");
        println!();

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
        crate::common::input_network_name(&(context.config.clone(),))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(skip_default_from_cli)]
/// How would you like to proceed
pub enum Submit {
    #[strum_discriminants(strum(message = "create      - Create a new account"))]
    Create,
}

impl interactive_clap::FromCli for Submit {
    type FromCliContext = NetworkContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let clap_variant = match optional_clap_variant {
            Some(CliSubmit::Create) => interactive_clap::ResultFromCli::Ok(CliSubmit::Create),
            None => Self::choose_variant(context.clone()),
        };

        if let interactive_clap::ResultFromCli::Ok(_) = &clap_variant {
            let mut storage_message = String::new();
            match (context.on_after_getting_network_callback)(
                &context.network_config,
                &mut storage_message,
            ) {
                Ok(_) => (),
                Err(report) => {
                    return interactive_clap::ResultFromCli::Err(
                        optional_clap_variant,
                        color_eyre::Report::msg(report),
                    )
                }
            };
            match (context.on_before_creating_account_callback)(
                &context.network_config,
                &context.new_account_id,
                &context.public_key,
            ) {
                Ok(_) => (),
                Err(report) => {
                    return interactive_clap::ResultFromCli::Err(
                        optional_clap_variant,
                        color_eyre::Report::msg(report),
                    )
                }
            };
            println!("{storage_message}");
        };
        clap_variant
    }
}

pub type OnAfterGettingNetworkCallback =
    std::sync::Arc<dyn Fn(&crate::config::NetworkConfig, &mut String) -> crate::CliResult>;

pub type OnBeforeCreatingAccountCallback = std::sync::Arc<
    dyn Fn(
        &crate::config::NetworkConfig,
        &crate::types::account_id::AccountId,
        &near_crypto::PublicKey,
    ) -> crate::CliResult,
>;
