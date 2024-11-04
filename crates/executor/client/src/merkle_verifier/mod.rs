use alloy_primitives::Bytes;
use alloy_primitives::FixedBytes;
use alloy_sol_types::sol_data;
use alloy_sol_types::SolType;
use alloy_trie::proof::ProofVerificationError;
use alloy_trie::{proof::verify_proof, Nibbles};

pub(crate) struct MerkleVerifier {}

#[derive(Debug)]
pub(crate) enum MerkleVerifierError {
    FailedToVerifyProof(ProofVerificationError),
    FailedToDecodeParams,
    FailedToLoad,
}

impl MerkleVerifier {
    pub(crate) fn verify_proof(
        txn_data: Bytes,
        receipt_root: FixedBytes<32>,
        proof_data: Bytes,
    ) -> Result<bool, MerkleVerifierError> {
        type MerklePatriciaProofVerifyParams = (sol_data::Bytes, sol_data::Array<sol_data::Bytes>);

        match MerklePatriciaProofVerifyParams::abi_decode_sequence(proof_data.as_ref(), true) {
            Ok(params) => {
                let key_nibbles = Nibbles::from_vec(params.0.to_vec());

                // rlp serialized receipt, that weird serialized data
                let expected_value = txn_data.to_vec();

                let proof = params.1.iter();

                match verify_proof(
                    receipt_root,
                    key_nibbles,
                    Some(expected_value.clone()),
                    proof,
                ) {
                    Ok(_) => Ok(true),
                    Err(e) => Err(MerkleVerifierError::FailedToVerifyProof(e)),
                }
            }
            Err(_e) => Err(MerkleVerifierError::FailedToDecodeParams),
        }
    }
}
