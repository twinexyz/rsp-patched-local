#![no_main]
sp1_zkvm::entrypoint!(main);

use rsp_client_executor::{io::ClientExecutorInput, ClientExecutor, DevnetVarient};
// use revm_primitives::FixedBytes;


pub fn main() {
    // Read the input.
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<Vec<ClientExecutorInput>>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    for i in input {
        executor.execute::<DevnetVarient>(i).expect("failed to execute client");
    }
    // let block = executor_output.block;
    let hash_vector = Vec::<u8>::new();
    // let block_number = FixedBytes::from(block.number);
    // let mut block_number = Vec::from(block_number.as_slice());
    // hash_vector.append(&mut block_number);

    // let mut block_hash = Vec::from(block.hash_slow().as_slice());
    // hash_vector.append(&mut block_hash);

    // let mut previous_state_root = Vec::from(input.previous_state_root.as_slice());
    // hash_vector.append(&mut previous_state_root); 

    // let mut state_root = Vec::from(block.state_root.as_slice()); 
    // hash_vector.append(&mut state_root);

    // let mut txn_root = Vec::from(block.transactions_root.as_slice());
    // hash_vector.append(&mut txn_root);

    // let mut receipt_root = Vec::from(block.receipts_root.as_slice());
    // hash_vector.append(&mut receipt_root);


    sp1_zkvm::io::commit_slice(&hash_vector);
}
