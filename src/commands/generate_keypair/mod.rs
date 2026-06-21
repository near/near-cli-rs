use color_eyre::eyre::Context;
use inquire::{CustomType, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

/// Generate a fresh key pair offline, without touching any account or the
/// network. Supports the classic Ed25519 keys as well as the post-quantum
/// ML-DSA-65 signature scheme (FIPS 204).
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypairCommand {
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// Which signature scheme should the new key pair use?
    signature_scheme: SignatureScheme,
    #[interactive_clap(subcommand)]
    output_mode: OutputMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    key_pair: GeneratedKeyPair,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<GenerateKeypairCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair = GeneratedKeyPair::generate(&scope.signature_scheme)?;
        Ok(Self { key_pair })
    }
}

impl GenerateKeypairCommand {
    fn input_signature_scheme(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<SignatureScheme>> {
        let variants = SignatureSchemeDiscriminants::iter().collect::<Vec<_>>();
        let selected = Select::new(
            "Which signature scheme should the new key pair use?",
            variants,
        )
        .prompt()?;
        Ok(Some(match selected {
            SignatureSchemeDiscriminants::Ed25519 => SignatureScheme::Ed25519,
            SignatureSchemeDiscriminants::MlDsa65 => SignatureScheme::MlDsa65,
        }))
    }
}

#[derive(Debug, Clone, EnumDiscriminants, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Which signature scheme should the new key pair use?
pub enum SignatureScheme {
    /// Ed25519 (classic, default NEAR key type)
    #[value(name = "ed25519")]
    Ed25519,
    /// ML-DSA-65 post-quantum signature scheme (FIPS 204)
    #[value(name = "ml-dsa-65")]
    MlDsa65,
}

impl interactive_clap::ToCli for SignatureScheme {
    type CliVariant = SignatureScheme;
}

impl std::fmt::Display for SignatureScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ed25519"),
            Self::MlDsa65 => write!(f, "ml-dsa-65"),
        }
    }
}

impl std::str::FromStr for SignatureScheme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ed25519" => Ok(Self::Ed25519),
            "ml-dsa-65" => Ok(Self::MlDsa65),
            _ => Err(format!("SignatureScheme: invalid value `{s}`")),
        }
    }
}

impl std::fmt::Display for SignatureSchemeDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ed25519    - Ed25519 (classic, default NEAR key type)"),
            Self::MlDsa65 => write!(
                f,
                "ml-dsa-65  - ML-DSA-65 post-quantum signature scheme (FIPS 204)"
            ),
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GenerateKeypairContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to output the generated key pair?
pub enum OutputMode {
    #[strum_discriminants(strum(
        message = "print-to-terminal  - Print the generated key pair to the terminal"
    ))]
    /// Print the generated key pair to the terminal
    PrintToTerminal(PrintToTerminal),
    #[strum_discriminants(strum(
        message = "save-to-file       - Save the generated key pair to a JSON file"
    ))]
    /// Save the generated key pair to a JSON file
    SaveToFile(SaveToFile),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = PrintToTerminalContext)]
pub struct PrintToTerminal;

#[derive(Debug, Clone)]
pub struct PrintToTerminalContext;

impl PrintToTerminalContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        _scope: &<PrintToTerminal as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        previous_context.key_pair.print_to_terminal();
        Ok(Self)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = SaveToFileContext)]
pub struct SaveToFile {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the path to the JSON file where the key pair should be saved (e.g. ./my-key.json)?
    file_path: crate::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SaveToFileContext;

impl SaveToFileContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveToFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.file_path.clone().into();
        previous_context.key_pair.save_to_file(&file_path)?;
        Ok(Self)
    }
}

impl SaveToFile {
    fn input_file_path(
        _context: &GenerateKeypairContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new(
                "What is the path to the JSON file where the key pair should be saved?",
            )
            .with_starting_input("key.json")
            .prompt()?,
        ))
    }
}

/// A freshly generated key pair, holding everything we want to display or
/// persist for the chosen signature scheme.
#[derive(Debug, Clone)]
enum GeneratedKeyPair {
    Ed25519(crate::common::KeyPairProperties),
    /// ML-DSA-65 keys are random (no seed phrase / HD derivation) and have no
    /// implicit-account-id form defined yet, so we only keep the key strings.
    MlDsa65 {
        public_key: String,
        private_key: String,
    },
}

impl GeneratedKeyPair {
    fn generate(signature_scheme: &SignatureScheme) -> color_eyre::eyre::Result<Self> {
        match signature_scheme {
            SignatureScheme::Ed25519 => Ok(Self::Ed25519(crate::common::generate_keypair()?)),
            SignatureScheme::MlDsa65 => {
                let private_key =
                    near_crypto::SecretKey::from_random(near_crypto::KeyType::MLDSA65);
                let public_key = private_key.public_key();
                Ok(Self::MlDsa65 {
                    public_key: public_key.to_string(),
                    private_key: private_key.to_string(),
                })
            }
        }
    }

    fn print_to_terminal(&self) {
        match self {
            Self::Ed25519(properties) => {
                eprintln!(
                    "\nSignature scheme: Ed25519\
                     \nMaster Seed Phrase: {}\
                     \nSeed Phrase HD Path: {}\
                     \nImplicit Account ID: {}\
                     \nPublic Key: {}\
                     \nSECRET KEYPAIR: {}",
                    properties.master_seed_phrase,
                    properties.seed_phrase_hd_path,
                    properties.implicit_account_id,
                    properties.public_key_str,
                    properties.secret_keypair_str,
                );
            }
            Self::MlDsa65 {
                public_key,
                private_key,
            } => {
                eprintln!(
                    "\nSignature scheme: ML-DSA-65 (post-quantum, FIPS 204)\
                     \nPublic Key: {public_key}\
                     \nSECRET KEYPAIR: {private_key}"
                );
            }
        }
    }

    fn save_to_file(&self, file_path: &std::path::Path) -> color_eyre::eyre::Result<()> {
        if let Some(parent) = file_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory for {file_path:?}"))?;
        }
        let json = match self {
            Self::Ed25519(properties) => serde_json::to_string_pretty(properties)?,
            Self::MlDsa65 {
                public_key,
                private_key,
            } => serde_json::to_string_pretty(&serde_json::json!({
                "public_key": public_key,
                "private_key": private_key,
            }))?,
        };
        std::fs::write(file_path, json)
            .wrap_err_with(|| format!("Failed to write the key pair to {file_path:?}"))?;
        eprintln!("\nThe key pair was saved to {file_path:?}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn ml_dsa_65_keypair_roundtrips() {
        let key_pair = GeneratedKeyPair::generate(&SignatureScheme::MlDsa65).unwrap();
        let GeneratedKeyPair::MlDsa65 {
            public_key,
            private_key,
        } = &key_pair
        else {
            panic!("expected an ML-DSA-65 key pair");
        };
        assert!(public_key.starts_with("ml-dsa-65:"), "{public_key}");
        assert!(private_key.starts_with("ml-dsa-65:"), "{private_key}");

        // The printed strings must parse back into near_crypto types of the
        // post-quantum key type, and the secret key must derive the public key.
        let parsed_public = near_crypto::PublicKey::from_str(public_key).unwrap();
        let parsed_secret = near_crypto::SecretKey::from_str(private_key).unwrap();
        // `KeyType` intentionally does not implement `PartialEq` in 2.13, so we
        // match on the variant instead of comparing with `assert_eq!`.
        assert!(matches!(
            parsed_public.key_type(),
            near_crypto::KeyType::MLDSA65
        ));
        assert!(matches!(
            parsed_secret.key_type(),
            near_crypto::KeyType::MLDSA65
        ));
        assert_eq!(parsed_secret.public_key(), parsed_public);

        // A signature produced by the secret key must verify under the public key.
        let message = b"post-quantum near-cli-rs";
        let signature = parsed_secret.sign(message);
        assert!(signature.verify(message, &parsed_public));
    }

    #[test]
    fn ed25519_remains_the_classic_default() {
        let key_pair = GeneratedKeyPair::generate(&SignatureScheme::Ed25519).unwrap();
        let GeneratedKeyPair::Ed25519(properties) = &key_pair else {
            panic!("expected an Ed25519 key pair");
        };
        assert!(properties.public_key_str.starts_with("ed25519:"));
        assert!(near_crypto::PublicKey::from_str(&properties.public_key_str).is_ok());
    }
}
