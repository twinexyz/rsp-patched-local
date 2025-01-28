#![no_main]
sp1_zkvm::entrypoint!(main);

use rsp_client_executor::{io::ClientExecutorInput, ChainVariant, ClientExecutor};

pub fn main() {
    // Read the input.
    let input = sp1_zkvm::io::read_vec();
    let input = bincode::deserialize::<Vec<ClientExecutorInput>>(&input).unwrap();

    // Execute the block.
    let executor = ClientExecutor;
    for i in input {
        executor.execute(i, &ChainVariant::mainnet()).expect("failed to execute client");
    }
}
