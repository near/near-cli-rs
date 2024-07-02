use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `proposals` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ProposalsArgs {
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ProposalsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "validator".to_string(),
            "proposals".to_string(),
            "network-config".to_string(),
            network_id,
        ];

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
 
    #[test]
    fn proposals_testnet() {
      let network_id = "testnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let proposals_args = ProposalsArgs::parse_from(&[
                "near",
                network_id_parameter_alias,
                network_id
            ]);
            let result = ProposalsArgs::to_cli_args(&proposals_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "validator proposals network-config {}",
                    network_id,
                )
            );
        }
    }

    #[test]
    fn proposals_mainnet() {
      let network_id = "mainnet";
      let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[0]);
      let proposals_args = ProposalsArgs::parse_from(&[
          "near",
          network_id_parameter_alias,
          network_id
      ]);
      let result = ProposalsArgs::to_cli_args(&proposals_args, "testnet".to_string());
      assert_eq!(
          result.join(" "),
          format!(
              "validator proposals network-config {}",
              network_id,
          )
      );
    }
}