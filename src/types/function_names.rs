#[derive(Debug, Default, Clone)]
pub struct FunctionNames(pub Vec<String>);

impl FunctionNames {
    pub fn parse_restricted(input: &str) -> color_eyre::eyre::Result<Self> {
        color_eyre::eyre::ensure!(
            !input.contains('"'),
            "Function names must not contain double quotes"
        );

        let function_names = input
            .split(',')
            .map(|name| name.trim().to_string())
            .collect::<Vec<_>>();
        color_eyre::eyre::ensure!(
            function_names.iter().all(|name| !name.is_empty()),
            "Function names must not contain empty entries"
        );

        Ok(Self(function_names))
    }
}

impl std::fmt::Display for FunctionNames {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

impl std::str::FromStr for FunctionNames {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Ok(Self(vec![]));
        }
        Self::parse_restricted(input)
    }
}

impl From<FunctionNames> for Vec<String> {
    fn from(item: FunctionNames) -> Self {
        item.0
    }
}

impl From<Vec<String>> for FunctionNames {
    fn from(item: Vec<String>) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for FunctionNames {
    type CliVariant = FunctionNames;
}

#[cfg(test)]
mod tests {
    use super::FunctionNames;

    #[test]
    fn parses_function_names() {
        let function_names: FunctionNames = "storage_deposit, ft_transfer".parse().unwrap();

        assert_eq!(function_names.0, vec!["storage_deposit", "ft_transfer"]);
    }

    #[test]
    fn parses_empty_function_names_as_unrestricted() {
        assert!("".parse::<FunctionNames>().unwrap().0.is_empty());
    }

    #[test]
    fn rejects_quoted_function_names() {
        assert!("\"ft_transfer\"".parse::<FunctionNames>().is_err());
    }

    #[test]
    fn rejects_empty_function_name_entries() {
        assert!("  ".parse::<FunctionNames>().is_err());
        assert!("ft_transfer,".parse::<FunctionNames>().is_err());
        assert!(",ft_transfer".parse::<FunctionNames>().is_err());
    }

    #[test]
    fn restricted_parser_rejects_an_empty_list() {
        assert!(FunctionNames::parse_restricted("").is_err());
    }
}
