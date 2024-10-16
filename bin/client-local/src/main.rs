#![no_main]
sp1_zkvm::entrypoint!(main);

use rsp_client_executor::{io::ClientExecutorInput, ClientExecutor, DevnetVarient};
use revm_primitives::FixedBytes;


pub fn main() {
    // Read the input.
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<ClientExecutorInput>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    let block = executor.execute::<DevnetVarient>(input).expect("failed to execute client");
    let mut hash_vector = Vec::<u8>::new();
    let block_number = FixedBytes::from(block.number);
    let mut block_number = block_number.as_slice();
    hash_vector.append(&mut block_number);
    
    let mut state_root = block.state_root.as_slice(); 
    hash_vector.append(&mut state_root);

    for txn in block.body {
        let mut txn_hash = Vec::from(txn.hash.as_slice());
        hash_vector.append(&mut txn_hash);
    }
    sp1_zkvm::io::commit_slice(&hash_vector);
}
