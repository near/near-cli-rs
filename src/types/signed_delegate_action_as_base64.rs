use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone)]
pub struct SignedDelegateActionAsBase64 {
    pub inner: near_primitives::delegate_action::SignedDelegateAction,
}

impl std::str::FromStr for SignedDelegateActionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::delegate_action::SignedDelegateAction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {}", err))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for SignedDelegateActionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signed_delegate_action = near_primitives::serialize::to_base64(
            self.inner
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        write!(f, "{}", base64_signed_delegate_action)
    }
}

impl interactive_clap::ToCli for SignedDelegateActionAsBase64 {
    type CliVariant = SignedDelegateActionAsBase64;
}