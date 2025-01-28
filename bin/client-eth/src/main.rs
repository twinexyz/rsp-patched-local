#![no_main]
sp1_zkvm::entrypoint!(main);

use rsp_client_executor::{io::ClientExecutorInput, ClientExecutor, EthereumVariant};

pub fn main() {
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<Vec<ClientExecutorInput>>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    let mut executor_outputs = Vec::new();
    for i in input {
        let output = executor.execute::<EthereumVariant>(i).expect("failed to execute client");
        executor_outputs.push(output);
    }

    // Commit the block hash.
    // sp1_zkvm::io::commit(&block_hash);
}
