#![no_main]
sp1_zkvm::entrypoint!(main);

use rsp_client_executor::{io::ClientExecutorInput, ClientExecutor, DevnetVarient};

pub fn main() {
    // Read the input.
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<ClientExecutorInput>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    let header = executor.execute::<DevnetVarient>(input).expect("failed to execute client");
    let block_hash = header.hash_slow();
    let transaction_root = header.transactions_root;
    // Commit the block hash.
    sp1_zkvm::io::commit(&transaction_root);
}
