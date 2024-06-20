use ed25519_dalek::VerifyingKey;
use olaf::{simplpedpop::AllMessage, SigningKeypair};
use std::{
    fs::{self, File},
    io::Write,
};

use crate::types::path_buf::PathBuf;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SimplpedpopRound1Context)]
pub struct SimplpedpopRound1 {
    #[interactive_clap(long)]
    /// The threshold for the SimplPedPoP protocol
    threshold: u64,
    #[interactive_clap(long)]
    /// The folder that contains the files for the round 1 of the SimplPedPoP protocol
    files: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SimplpedpopRound1Context;

impl SimplpedpopRound1Context {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<SimplpedpopRound1 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.files.clone().into();

        let secret_key_string = fs::read_to_string(file_path.join("contributor_secret_key.json"))?;

        let encoded_string: String = serde_json::from_str(&secret_key_string).unwrap();

        let s = bs58::decode(encoded_string).into_vec().unwrap();

        let mut secret_key_bytes = [0; 32];
        secret_key_bytes.copy_from_slice(&s);

        let mut keypair = SigningKeypair::from_secret_key(&secret_key_bytes);

        let recipients_string = fs::read_to_string(file_path.join("recipients.json")).unwrap();

        let encoded_strings: Vec<String> = serde_json::from_str(&recipients_string).unwrap();

        let recipients: Vec<VerifyingKey> = encoded_strings
            .iter()
            .map(|encoded_string| {
                let s = bs58::decode(encoded_string).into_vec().unwrap();
                let mut recipient = [0; 32];
                recipient.copy_from_slice(&s);
                VerifyingKey::from_bytes(&recipient).unwrap()
            })
            .collect();

        let all_message: AllMessage = keypair
            .simplpedpop_contribute_all(scope.threshold as u16, recipients)
            .unwrap();

        let all_message_bytes: Vec<u8> = all_message.to_bytes();
        let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];

        let all_message_json = serde_json::to_string_pretty(&all_message_vec)?;

        let mut all_message_file = File::create(file_path.join("all_messages.json")).unwrap();

        all_message_file.write_all(all_message_json.as_bytes())?;

        Ok(Self)
    }
}
