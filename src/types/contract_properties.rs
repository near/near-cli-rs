use near_verify_rs::types::{contract_source_metadata::Standard, sha256_checksum::SHA256Checksum};

pub struct ContractProperties {
    pub code: Vec<u8>,
    pub hash: SHA256Checksum,
    pub version: Option<String>,
    pub standards: Vec<Standard>,
    pub link: Option<String>,
    pub source: String,
    pub build_environment: String,
    pub build_command: Vec<String>,
}

impl std::fmt::Display for ContractProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contract code hash: {}\nContract version:\t{}\nStandards used by the contract:\t[{}]\nView the contract's source code on:\t{}\nBuild Environment:\t{}\nBuild Command:\t{}",
            self.hash.to_base58_string(),
            self.version.clone().unwrap_or("N/A".to_string()),
            self.standards.iter().map(|standard| format!("{}:{}", standard.standard, standard.version)).collect::<Vec<String>>().join(", "),
            if let Some(link) = &self.link {
                link
            } else {
                &self.source
            },
            self.build_environment,
            shell_words::join(self.build_command.clone()),
        )
    }
}
