#![no_main]
sp1_zkvm::entrypoint!(main);

use revm_primitives::{keccak256, FixedBytes};
use rsp_client_executor::{io::ClientExecutorInput, BlockInfo, ChainVariant, ClientExecutor, PublicCommitment};
// use revm_primitives::FixedBytes;


pub fn main() {
    // Read the input.
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<ClientExecutorInput>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    // let mut executor_outputs = Vec::new();
    // for i in input {
    //     let output = executor.execute(i, &ChainVariant::devnet()).expect("failed to execute client");
    //     executor_outputs.push(output);
    // }

    let output = executor.execute(input, &ChainVariant::devnet()).expect("failed to execute client");

    // let mut pub_commitment_slice = Vec::new();
    // let from_block = executor_outputs.first().expect("empty output").number;
    // let to_block = executor_outputs.last().expect("empty outputs").number;
    
    // () = executor_outputs.iter().map(|output| {
    //     let public_commitment = BlockInfo {
    //         previous_block: FixedBytes::from_slice(&output.parent_hash.0),
    //         block_hash: FixedBytes::from_slice(&output.hash_slow().0),
    //         transaction_root: FixedBytes::from_slice(&output.transactions_root.0),
    //         receipt_root: FixedBytes::from_slice(&output.receipts_root.0),
    //     };
    //     let mut public_commitment = public_commitment.abi_encode_packed();
    //     pub_commitment_slice.append(&mut public_commitment);
    // }).collect();
    

    // let public_commitment = PublicCommitment {
    //     from_block,
    //     to_block,
    //     batch_hash: keccak256(pub_commitment_slice),
    // };

    // sp1_zkvm::io::commit_slice(&public_commitment.abi_encode_packed());
}
