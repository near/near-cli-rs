#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ReceiverValidation {
    Check,
    Skip,
}

impl interactive_clap::ToCli for ReceiverValidation {
    type CliVariant = Self;
}

pub(super) fn input() -> color_eyre::eyre::Result<Option<ReceiverValidation>> {
    tracing_indicatif::suspend_tracing_indicatif(|| {
        match inquire::Select::new(
            "Do you want to check the receiver account?",
            vec![ReceiverValidation::Check, ReceiverValidation::Skip],
        )
        .with_help_message("Skip the lookup when funding a fresh implicit account.")
        .prompt()
        {
            Ok(choice) => Ok(Some(choice)),
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => Ok(None),
            Err(err) => Err(err.into()),
        }
    })
}

pub(super) fn validate(
    network_config: &crate::config::NetworkConfig,
    receiver_account_id: &near_primitives::types::AccountId,
    verbosity: crate::Verbosity,
    offline: bool,
    choice: Option<ReceiverValidation>,
    interactive: bool,
) -> color_eyre::eyre::Result<bool> {
    match choice {
        Some(ReceiverValidation::Skip) => Ok(true),
        None => crate::common::validate_receiver_account_id(
            network_config,
            receiver_account_id,
            verbosity,
            offline,
        ),
        Some(ReceiverValidation::Check) => {
            color_eyre::eyre::ensure!(
                !offline,
                "Cannot check the receiver account in offline mode. Omit --receiver-validation to keep the offline behavior, or use --receiver-validation skip."
            );
            // An explicit check also works in quiet mode. Scripted checks fail instead
            // of opening a confirmation dialog when validation warns.
            crate::common::validate_receiver_account_id_with_warning(
                network_config,
                receiver_account_id,
                crate::Verbosity::Interactive,
                false,
                |message| {
                    if interactive {
                        crate::common::handle_validation_warning(message)
                    } else {
                        color_eyre::eyre::bail!(
                            "{message} Use --receiver-validation skip to proceed without checking."
                        );
                    }
                },
            )
        }
    }
}
