use base64::{Engine as _, engine::general_purpose::STANDARD};
use borsh::BorshDeserialize;

#[derive(Debug, Clone)]
pub struct SignedDelegateActionAsBase64 {
    inner: near_kit::SignedDelegateAction,
}

impl serde::Serialize for SignedDelegateActionAsBase64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let signed_delegate_action_borsh = borsh::to_vec(&self.inner).map_err(|err| {
            serde::ser::Error::custom(format!(
                "The value could not be borsh encoded due to: {err}"
            ))
        })?;
        let signed_delegate_action_as_base64 =
            STANDARD.encode(&signed_delegate_action_borsh);
        serializer.serialize_str(&signed_delegate_action_as_base64)
    }
}

impl<'de> serde::Deserialize<'de> for SignedDelegateActionAsBase64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let signed_delegate_action_as_base64 =
            <String as serde::Deserialize>::deserialize(deserializer)?;
        let signed_delegate_action_borsh = STANDARD.decode(
            &signed_delegate_action_as_base64,
        )
        .map_err(|err| {
            serde::de::Error::custom(format!(
                "The value could not decoded from base64 due to: {err}"
            ))
        })?;
        let signed_delegate_action = near_kit::SignedDelegateAction::deserialize(
            &mut &signed_delegate_action_borsh[..],
        )
        .map_err(|err| {
            serde::de::Error::custom(format!(
                "The value could not decoded from borsh due to: {err}"
            ))
        })?;
        Ok(Self {
            inner: signed_delegate_action,
        })
    }
}

impl std::str::FromStr for SignedDelegateActionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = STANDARD.decode(s)
            .map_err(|err| format!("parsing of signed delegate action failed due to base64 sequence being invalid: {err}"))?;
        let inner = near_kit::SignedDelegateAction::deserialize(&mut &bytes[..])
            .map_err(|err| format!("delegate action could not be deserialized from borsh: {err}"))?;
        Ok(Self { inner })
    }
}

impl std::fmt::Display for SignedDelegateActionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signed_delegate_action = STANDARD.encode(
            &borsh::to_vec(&self.inner)
                .expect("Signed Delegate Action serialization to borsh is not expected to fail"),
        );
        write!(f, "{base64_signed_delegate_action}")
    }
}

impl interactive_clap::ToCli for SignedDelegateActionAsBase64 {
    type CliVariant = SignedDelegateActionAsBase64;
}

impl From<near_kit::SignedDelegateAction>
    for SignedDelegateActionAsBase64
{
    fn from(value: near_kit::SignedDelegateAction) -> Self {
        Self { inner: value }
    }
}

impl From<SignedDelegateActionAsBase64>
    for near_kit::SignedDelegateAction
{
    fn from(signed_delegate_action: SignedDelegateActionAsBase64) -> Self {
        signed_delegate_action.inner
    }
}
