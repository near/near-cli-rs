#[derive(Debug, Default, Clone)]
pub struct VecString(pub Vec<String>);

impl std::fmt::Display for VecString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

impl std::str::FromStr for VecString {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self(vec![]));
        }
        let vec_str: Vec<String> = s.split(',').map(|str| str.trim().to_string()).collect();
        Ok(Self(vec_str))
    }
}

impl VecString {
    pub fn parse_restricted_function_names(input: &str) -> color_eyre::eyre::Result<Self> {
        color_eyre::eyre::ensure!(
            !input.contains('"'),
            "Function names must not contain double quotes"
        );

        let function_names: Self = input.parse()?;
        color_eyre::eyre::ensure!(
            !function_names.0.is_empty() && function_names.0.iter().all(|name| !name.is_empty()),
            "Enter at least one function name, or choose unrestricted function access"
        );

        Ok(function_names)
    }
}

impl From<VecString> for Vec<String> {
    fn from(item: VecString) -> Self {
        item.0
    }
}

impl From<Vec<String>> for VecString {
    fn from(item: Vec<String>) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for VecString {
    type CliVariant = VecString;
}

#[cfg(test)]
mod tests {
    use super::VecString;

    #[test]
    fn parses_restricted_function_names() {
        let function_names =
            VecString::parse_restricted_function_names("storage_deposit, ft_transfer").unwrap();

        assert_eq!(function_names.0, vec!["storage_deposit", "ft_transfer"]);
    }

    #[test]
    fn rejects_quoted_function_names() {
        assert!(VecString::parse_restricted_function_names("\"ft_transfer\"").is_err());
    }

    #[test]
    fn rejects_empty_function_names() {
        assert!(VecString::parse_restricted_function_names("").is_err());
        assert!(VecString::parse_restricted_function_names("  ").is_err());
        assert!(VecString::parse_restricted_function_names("ft_transfer,").is_err());
    }
}
