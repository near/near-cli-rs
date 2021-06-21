use std::io::Write;
use std::str::FromStr;

fn bip32path_to_string(bip32path: &slip10::BIP32Path) -> String {
    const HARDEND: u32 = 1 << 31;

    format!(
        "m/{}",
        (0..bip32path.depth())
            .map(|index| {
                let value = *bip32path.index(index).unwrap();
                if value < HARDEND {
                    value.to_string()
                } else {
                    format!("{}'", value - HARDEND)
                }
            })
            .collect::<Vec<String>>()
            .join("/")
    )
}

/// Generate a key pair of secret and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, Default, clap::Clap)]
pub struct CliGenerateKeypair {
    #[clap(subcommand)]
    permission: Option<super::add_access_key::CliAccessKeyPermission>,
}

#[derive(Debug)]
pub struct GenerateKeypair {
    pub permission: super::add_access_key::AccessKeyPermission,
}

impl From<CliGenerateKeypair> for GenerateKeypair {
    fn from(item: CliGenerateKeypair) -> Self {
        let permission: super::add_access_key::AccessKeyPermission = match item.permission {
            Some(cli_permission) => {
                super::add_access_key::AccessKeyPermission::from(cli_permission)
            }
            None => super::add_access_key::AccessKeyPermission::choose_permission(),
        };
        Self { permission }
    }
}

impl GenerateKeypair {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let new_master_seed_phrase_words_count: usize = 12;
        let seed_phrase_hd_path = slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();

        let (master_seed_phrase, master_seed) = {
            let mnemonic = bip39::Mnemonic::generate(new_master_seed_phrase_words_count)?;
            let mut master_seed_phrase = String::new();
            for (index, word) in mnemonic.word_iter().enumerate() {
                if index != 0 {
                    master_seed_phrase.push(' ');
                }
                master_seed_phrase.push_str(word);
            }
            (master_seed_phrase, mnemonic.to_seed(""))
        };
        let derived_private_key = slip10::derive_key_from_path(
            &master_seed,
            slip10::Curve::Ed25519,
            &seed_phrase_hd_path,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to derive a key from the master key: {}",
                err
            ))
        })?;

        let secret_keypair = {
            let secret = ed25519_dalek::SecretKey::from_bytes(&derived_private_key.key)?;
            let public = ed25519_dalek::PublicKey::from(&secret);
            ed25519_dalek::Keypair { secret, public }
        };
        let implicit_account_id = hex::encode(&secret_keypair.public);
        let public_key_str = format!(
            "ed25519:{}",
            bs58::encode(&secret_keypair.public).into_string()
        );
        let secret_keypair_str = format!(
            "ed25519:{}",
            bs58::encode(&secret_keypair.to_bytes()).into_string()
        );
        let public_key = near_crypto::PublicKey::from_str(&public_key_str)?;

        let buf = format!(
            "{}",
            serde_json::json!({
            "master_seed_phrase": master_seed_phrase,
            "seed_phrase_hd_path": bip32path_to_string(&seed_phrase_hd_path),
            "account_id": implicit_account_id,
            "public_key": public_key_str,
            "private_key": secret_keypair_str,
            })
        );
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let file_name: std::path::PathBuf =
            format!("{}.json", &prepopulated_unsigned_transaction.receiver_id).into();
        let mut path = std::path::PathBuf::from(&home_dir);
        path.push(crate::consts::DIR_NAME_KEY_CHAIN);
        std::fs::create_dir_all(&path)?;
        path.push(file_name);
        if path.exists() {
            return Err(color_eyre::Report::msg(format!(
                "The file: {} already exists!",
                &path.display()
            )));
        };
        std::fs::File::create(&path)
            .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
            .write(buf.as_bytes())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
            })?;
        println!(
            "The data for the access key is saved in a file {}",
            &path.display()
        );

        match self.permission {
            super::add_access_key::AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        0,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        public_key,
                    )
                    .await
            }
            super::add_access_key::AccessKeyPermission::GrantFunctionCallAccess(
                function_call_type,
            ) => {
                function_call_type
                    .process(
                        0,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        public_key,
                    )
                    .await
            }
        }
    }
}
