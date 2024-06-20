use bip39::rand_core::OsRng;
use olaf::SigningKeypair;
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
};

use crate::types::path_buf::PathBuf;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = FrostRound1Context)]
pub struct FrostRound1 {
    #[interactive_clap(long)]
    /// The folder that contains the files for the round 1 of the FROST protocol
    files: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FrostRound1Context;

impl FrostRound1Context {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<FrostRound1 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.files.clone().into();

        let signing_share_string =
            fs::read_to_string(file_path.join("signing_share.json")).unwrap();

        let signing_share_vec: Vec<u8> = from_str(&signing_share_string).unwrap();

        let mut signing_share_bytes = [0; 64];
        signing_share_bytes.copy_from_slice(&signing_share_vec);

        let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

        let (signing_nonces, signing_commitments) = signing_share.commit(&mut OsRng);

        let signing_nonces_json =
            serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec()).unwrap();

        let mut signing_nonces_file = File::create(file_path.join("signing_nonces.json")).unwrap();

        signing_nonces_file
            .write_all(signing_nonces_json.as_bytes())
            .unwrap();

        let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];

        let signing_commitments_json =
            serde_json::to_string_pretty(&signing_commitments_vec).unwrap();

        let mut signing_commitments_file =
            File::create(file_path.join("signing_commitments.json")).unwrap();

        signing_commitments_file
            .write_all(signing_commitments_json.as_bytes())
            .unwrap();

        Ok(Self)
    }
}
