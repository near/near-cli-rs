use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

/// Generate a fresh key pair offline, without touching any account or the
/// network. Supports the classic Ed25519 keys as well as the post-quantum
/// ML-DSA-65 signature scheme (FIPS 204).
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypairCommand {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Which signature scheme should the new key pair use?
    signature_scheme: crate::common::SignatureScheme,
    #[interactive_clap(subcommand)]
    output_mode: OutputMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    key_pair: crate::common::GeneratedKeyPair,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<GenerateKeypairCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            key_pair: crate::common::GeneratedKeyPair::generate(&scope.signature_scheme)?,
        })
    }
}

impl GenerateKeypairCommand {
    fn input_signature_scheme(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::SignatureScheme>> {
        crate::common::input_signature_scheme()
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
        eprintln!("{}", previous_context.key_pair.terminal_info());
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
        if let Some(parent) = file_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory for {file_path:?}"))?;
        }
        std::fs::write(&file_path, previous_context.key_pair.keychain_json()?)
            .wrap_err_with(|| format!("Failed to write the key pair to {file_path:?}"))?;
        eprintln!("\nThe key pair was saved to {file_path:?}");
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

#[cfg(test)]
mod tests {
    use crate::common::{GeneratedKeyPair, SignatureScheme};
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
