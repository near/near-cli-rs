use near_primitives::{borsh, borsh::BorshDeserialize};

/// A signed delegate action of either meta-transaction version, carried in and
/// out of the CLI as a base64-encoded borsh blob (the form relayers accept).
///
/// * `V1` — NEP-366 [`SignedDelegateAction`], signed with an ordinary access key
///   (a plain nonce).
/// * `V2` — NEP-611 [`VersionedSignedDelegateAction`], signed with a gas key; its
///   nonce carries the parallel-nonce index that a V1 delegate action cannot
///   encode.
///
/// [`SignedDelegateAction`]: near_primitives::action::delegate::SignedDelegateAction
/// [`VersionedSignedDelegateAction`]: near_primitives::action::delegate::VersionedSignedDelegateAction
#[derive(Debug, Clone)]
pub enum SignedDelegateActionAsBase64 {
    V1(near_primitives::action::delegate::SignedDelegateAction),
    V2(near_primitives::action::delegate::VersionedSignedDelegateAction),
}

impl SignedDelegateActionAsBase64 {
    /// The account on whose behalf the delegated actions run (the meta-transaction sender).
    pub fn sender_id(&self) -> &near_primitives::types::AccountId {
        match self {
            Self::V1(signed_delegate_action) => &signed_delegate_action.delegate_action.sender_id,
            Self::V2(signed_delegate_action) => match &signed_delegate_action.delegate_action {
                near_primitives::action::delegate::VersionedDelegateActionPayload::V2(
                    delegate_action,
                ) => &delegate_action.sender_id,
            },
        }
    }

    /// Wrap the signed delegate action in the matching relay action
    /// (`Action::Delegate` for V1, `Action::DelegateV2` for V2).
    pub fn into_action(self) -> near_primitives::transaction::Action {
        match self {
            Self::V1(signed_delegate_action) => signed_delegate_action.into(),
            Self::V2(signed_delegate_action) => signed_delegate_action.into(),
        }
    }

    fn to_borsh(&self) -> std::io::Result<Vec<u8>> {
        match self {
            Self::V1(signed_delegate_action) => borsh::to_vec(signed_delegate_action),
            Self::V2(signed_delegate_action) => borsh::to_vec(signed_delegate_action),
        }
    }

    /// Decode a borsh blob into the right version. The two encodings can't
    /// collide: a V2 blob begins with the payload enum discriminant `0x00`,
    /// whereas a V1 blob begins with the borsh length prefix of the sender
    /// account id (a little-endian `u32` that is always >= 2). Try V1 first and
    /// fall back to V2.
    fn from_borsh(bytes: &[u8]) -> Result<Self, String> {
        if let Ok(signed_delegate_action) =
            near_primitives::action::delegate::SignedDelegateAction::try_from_slice(bytes)
        {
            return Ok(Self::V1(signed_delegate_action));
        }
        near_primitives::action::delegate::VersionedSignedDelegateAction::try_from_slice(bytes)
            .map(Self::V2)
            .map_err(|err| format!("delegate action could not be deserialized from borsh: {err}"))
    }
}

impl serde::Serialize for SignedDelegateActionAsBase64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let signed_delegate_action_borsh = self.to_borsh().map_err(|err| {
            serde::ser::Error::custom(format!(
                "The value could not be borsh encoded due to: {err}"
            ))
        })?;
        let signed_delegate_action_as_base64 =
            near_primitives::serialize::to_base64(&signed_delegate_action_borsh);
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
        let signed_delegate_action_borsh = near_primitives::serialize::from_base64(
            &signed_delegate_action_as_base64,
        )
        .map_err(|err| {
            serde::de::Error::custom(format!(
                "The value could not be decoded from base64 due to: {err}"
            ))
        })?;
        Self::from_borsh(&signed_delegate_action_borsh).map_err(serde::de::Error::custom)
    }
}

impl std::str::FromStr for SignedDelegateActionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let signed_delegate_action_borsh = near_primitives::serialize::from_base64(s).map_err(
            |err| format!("parsing of signed delegate action failed due to base64 sequence being invalid: {err}"),
        )?;
        Self::from_borsh(&signed_delegate_action_borsh)
    }
}

impl std::fmt::Display for SignedDelegateActionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_signed_delegate_action = near_primitives::serialize::to_base64(
            &self
                .to_borsh()
                .expect("Signed Delegate Action serialization to borsh is not expected to fail"),
        );
        write!(f, "{base64_signed_delegate_action}")
    }
}

impl interactive_clap::ToCli for SignedDelegateActionAsBase64 {
    type CliVariant = SignedDelegateActionAsBase64;
}

impl From<near_primitives::action::delegate::SignedDelegateAction>
    for SignedDelegateActionAsBase64
{
    fn from(value: near_primitives::action::delegate::SignedDelegateAction) -> Self {
        Self::V1(value)
    }
}

impl From<near_primitives::action::delegate::VersionedSignedDelegateAction>
    for SignedDelegateActionAsBase64
{
    fn from(value: near_primitives::action::delegate::VersionedSignedDelegateAction) -> Self {
        Self::V2(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_crypto::{KeyType, PublicKey, Signature};
    use near_primitives::action::delegate::{
        DelegateAction, DelegateActionV2, SignedDelegateAction, VersionedDelegateActionPayload,
        VersionedSignedDelegateAction,
    };

    fn sample_v1() -> SignedDelegateActionAsBase64 {
        SignedDelegateActionAsBase64::V1(SignedDelegateAction {
            delegate_action: DelegateAction {
                sender_id: "alice.near".parse().unwrap(),
                receiver_id: "bob.near".parse().unwrap(),
                actions: vec![],
                nonce: 1,
                max_block_height: 1000,
                public_key: PublicKey::empty(KeyType::ED25519),
            },
            signature: Signature::empty(KeyType::ED25519),
        })
    }

    fn sample_v2() -> SignedDelegateActionAsBase64 {
        SignedDelegateActionAsBase64::V2(VersionedSignedDelegateAction {
            delegate_action: VersionedDelegateActionPayload::V2(DelegateActionV2 {
                sender_id: "alice.near".parse().unwrap(),
                receiver_id: "bob.near".parse().unwrap(),
                actions: vec![],
                nonce: near_primitives::transaction::TransactionNonce::from_nonce_and_index(1, 3),
                max_block_height: 1000,
                public_key: PublicKey::empty(KeyType::ED25519),
            }),
            signature: Signature::empty(KeyType::ED25519),
        })
    }

    #[test]
    fn v1_base64_round_trips_as_v1() {
        let encoded = sample_v1().to_string();
        let decoded: SignedDelegateActionAsBase64 = encoded.parse().unwrap();
        assert!(
            matches!(decoded, SignedDelegateActionAsBase64::V1(_)),
            "a V1 blob must decode back to V1, not V2"
        );
        assert_eq!(decoded.to_string(), encoded);
    }

    #[test]
    fn v2_base64_round_trips_as_v2() {
        let encoded = sample_v2().to_string();
        let decoded: SignedDelegateActionAsBase64 = encoded.parse().unwrap();
        assert!(
            matches!(decoded, SignedDelegateActionAsBase64::V2(_)),
            "a V2 blob must decode back to V2, not V1"
        );
        assert_eq!(decoded.to_string(), encoded);
    }
}
