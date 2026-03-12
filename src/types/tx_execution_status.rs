#[derive(Debug, Clone)]
pub struct TxExecutionStatus(pub near_primitives::views::TxExecutionStatus);

impl From<TxExecutionStatus> for near_primitives::views::TxExecutionStatus {
    fn from(status: TxExecutionStatus) -> Self {
        status.0
    }
}

impl serde::Serialize for TxExecutionStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TxExecutionStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for TxExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            near_primitives::views::TxExecutionStatus::None => write!(f, "none"),
            near_primitives::views::TxExecutionStatus::Included => write!(f, "included"),
            near_primitives::views::TxExecutionStatus::ExecutedOptimistic => {
                write!(f, "executed-optimistic")
            }
            near_primitives::views::TxExecutionStatus::IncludedFinal => {
                write!(f, "included-final")
            }
            near_primitives::views::TxExecutionStatus::Executed => write!(f, "executed"),
            near_primitives::views::TxExecutionStatus::Final => write!(f, "final"),
        }
    }
}

impl std::str::FromStr for TxExecutionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self(near_primitives::views::TxExecutionStatus::None)),
            "included" => Ok(Self(near_primitives::views::TxExecutionStatus::Included)),
            "executed-optimistic" | "executed_optimistic" => Ok(Self(
                near_primitives::views::TxExecutionStatus::ExecutedOptimistic,
            )),
            "included-final" | "included_final" => Ok(Self(
                near_primitives::views::TxExecutionStatus::IncludedFinal,
            )),
            "executed" => Ok(Self(near_primitives::views::TxExecutionStatus::Executed)),
            "final" => Ok(Self(near_primitives::views::TxExecutionStatus::Final)),
            _ => Err(format!(
                "Unknown tx execution status: '{}'. Valid values: none, included, executed-optimistic, included-final, executed, final",
                s
            )),
        }
    }
}

impl interactive_clap::ToCli for TxExecutionStatus {
    type CliVariant = TxExecutionStatus;
}
