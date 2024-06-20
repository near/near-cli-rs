use olaf::{
    frost::{SigningCommitments, SigningNonces},
    simplpedpop::SPPOutput,
    SigningKeypair,
};
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
};

use crate::types::path_buf::PathBuf;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = FrostRound2Context)]
pub struct FrostRound2 {
    #[interactive_clap(long)]
    /// The folder that contains the files for the round 2 of the FROST protocol
    files: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FrostRound2Context;

impl FrostRound2Context {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<FrostRound2 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.files.clone().into();

        let signing_commitments_string =
            fs::read_to_string(file_path.join("signing_commitments.json")).unwrap();

        let signing_commitments_bytes: Vec<Vec<u8>> =
            from_str(&signing_commitments_string).unwrap();

        let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
            .iter()
            .map(|signing_commitments| SigningCommitments::from_bytes(signing_commitments).unwrap())
            .collect();

        let signing_nonces_string =
            fs::read_to_string(file_path.join("signing_nonces.json")).unwrap();

        let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string).unwrap();
        let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes).unwrap();

        let signing_share_string =
            fs::read_to_string(file_path.join("signing_share.json")).unwrap();

        let signing_share_vec: Vec<u8> = from_str(&signing_share_string).unwrap();

        let mut signing_share_bytes = [0; 64];
        signing_share_bytes.copy_from_slice(&signing_share_vec);

        let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

        let output_string = fs::read_to_string(file_path.join("spp_output.json")).unwrap();

        let output_bytes: Vec<u8> = from_str(&output_string).unwrap();
        let spp_output = SPPOutput::from_bytes(&output_bytes).unwrap();

        let tx_hash_string = fs::read_to_string(file_path.join("tx_hash.json")).unwrap();

        let tx_hash_str: String = from_str(&tx_hash_string).unwrap();

        let tx_hash_bytes = hex::decode(tx_hash_str).unwrap();

        let signing_package = signing_share
            .sign(
                &tx_hash_bytes,
                &spp_output,
                &signing_commitments,
                &signing_nonces,
            )
            .unwrap();

        let signing_packages_vec = vec![signing_package.to_bytes()];

        let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec).unwrap();

        let mut signing_package_file =
            File::create(file_path.join("signing_packages.json")).unwrap();

        signing_package_file
            .write_all(signing_package_json.as_bytes())
            .unwrap();

        Ok(Self)
    }
}
