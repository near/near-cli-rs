// TODO: remove copy + clone in favor of lifetimes

#[derive(Clone)]
pub struct Network {
    pub name: String,
}

impl std::str::FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Network {
            name: s.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct AccountId {
    pub id: String,
    pub network: String,
}

impl std::str::FromStr for AccountId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('@');
        let id = parts.next().ok_or("missing id")?;
        let network = parts.next().ok_or("missing network")?;
        Ok(AccountId {
            id: id.to_string(),
            network: network.to_string(),
        })
    }
}

pub fn convert(network: &Network) -> AccountId {
    AccountId {
        id: "welp".to_string(),
        network: network.name.clone(),
    }
}
