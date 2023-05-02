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
                .map_err(|err| format!("parsing of signed delegate action failed due to base64 sequence being invalid: {}", err))?,
            )
            .map_err(|err| format!("delegate action could not be deserialized from borsh: {}", err))?,
        })
    }
}

impl std::fmt::Display for SignedDelegateActionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signed_delegate_action = near_primitives::serialize::to_base64(
            self.inner
                .try_to_vec()
                .expect("Signed Delegate Action serialization to borsh is not expected to fail"),
        );
        write!(f, "{}", base64_signed_delegate_action)
    }
}

impl interactive_clap::ToCli for SignedDelegateActionAsBase64 {
    type CliVariant = SignedDelegateActionAsBase64;
}

impl From<near_primitives::delegate_action::SignedDelegateAction> for SignedDelegateActionAsBase64 {
    fn from(value: near_primitives::delegate_action::SignedDelegateAction) -> Self {
        Self { inner: value }
    }
}
