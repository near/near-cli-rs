use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl FromStr for Version {
    type Err = String;

    fn from_str(version_str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err("Failed to parse string as a version".to_string());
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| "Failed to parse major version")?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| "Failed to parse major version")?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| "Failed to parse major version")?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_cmp = self.major.cmp(&other.major);
        if major_cmp != Ordering::Equal {
            return major_cmp;
        }
        let minor_cmp = self.minor.cmp(&other.minor);
        if minor_cmp != Ordering::Equal {
            return minor_cmp;
        }
        self.patch.cmp(&other.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn string_to_version() {
        assert_eq!(
            "1.0.0".parse::<Version>().unwrap(),
            Version {
                major: 1,
                minor: 0,
                patch: 0
            }
        );
    }

    #[test]
    fn version_to_string() {
        assert_eq!(
            Version {
                major: 1,
                minor: 2,
                patch: 3
            }
            .to_string(),
            String::from("1.2.3")
        );
    }

    #[test]
    fn compare_version_by_patch() {
        let v1 = "1.0.0".parse::<Version>().unwrap();
        let v2 = "1.0.1".parse::<Version>().unwrap();

        assert!(v1 < v2);
    }

    #[test]
    fn compare_version_by_minor() {
        let v1 = "1.0.0".parse::<Version>().unwrap();
        let v2 = "1.1.0".parse::<Version>().unwrap();

        assert!(v1 < v2);
    }

    #[test]
    fn compare_version_by_major() {
        let v1 = "1.0.0".parse::<Version>().unwrap();
        let v2 = "2.0.0".parse::<Version>().unwrap();

        assert!(v1 < v2);
    }

    #[test]
    fn compare_equal_versions() {
        let v1 = "1.2.3".parse::<Version>().unwrap();
        let v2 = "1.2.3".parse::<Version>().unwrap();

        assert!(v1 == v2);
    }
}
