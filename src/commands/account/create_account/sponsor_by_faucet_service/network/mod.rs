#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::account::create_account::CreateAccountContext)]
#[interactive_clap(output_context = crate::transaction_signature_options::SubmitContext)]
pub struct Network {
    /// What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    pub submit: crate::transaction_signature_options::Submit,
}

#[derive(Clone)]
pub struct NetworkContext {
    config: crate::config::Config,
    account_properties: crate::commands::account::create_account::AccountProperties,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    network_config: crate::config::NetworkConfig,
}

impl NetworkContext {
    pub fn from_previous_context(
        previous_context: crate::commands::account::create_account::CreateAccountContext,
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
            "--", "create account:", &previous_context.account_properties.new_account_id
        );
        println!("{:>5} {:<20}", "--", "add access key:");
        println!(
            "{:>18} {:<13} {}",
            "", "public key:", &previous_context.account_properties.public_key
        );
        println!("{:>18} {:<13} FullAccess", "", "permission:");
        println!();

        Ok(Self {
            config: previous_context.config,
            account_properties: previous_context.account_properties,
            network_config,
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
        })
    }
}

impl From<NetworkContext> for crate::transaction_signature_options::SubmitContext {
    fn from(item: NetworkContext) -> Self {
        Self {
            network_config: item.network_config,
            submit_transaction:
                crate::transaction_signature_options::SubmitTransaction::SponsorService(
                    super::super::SponsorService {
                        account_properties: item.account_properties,
                    },
                ),
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl Network {
    fn input_network_name(
        context: &crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<String> {
        crate::common::input_network_name(&(context.config.clone(),))
    }
}
