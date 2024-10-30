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
    let executor_output = executor.execute::<DevnetVarient>(input.clone()).expect("failed to execute client");
    let block = executor_output.block;
    let mut hash_vector = Vec::<u8>::new();
    let block_number = FixedBytes::from(block.number);
    let mut block_number = Vec::from(block_number.as_slice());
    hash_vector.append(&mut block_number);
    
    let mut state_root = Vec::from(block.state_root.as_slice()); 
    hash_vector.append(&mut state_root);

    let mut txn_root = Vec::from(block.transactions_root.as_slice());
    hash_vector.append(&mut txn_root);

    let mut receipt_root = Vec::from(block.receipts_root.as_slice());
    hash_vector.append(&mut receipt_root);

    let mut deposit_transaction = Vec::from(block.body.first().unwrap().hash.as_slice());
    hash_vector.append(&mut deposit_transaction);

    let len = input.clone().withdrawal_txn_hashes.len().to_be_bytes();
    let len: FixedBytes<8> = FixedBytes::from_slice(&len);
    let mut len = Vec::from(len.as_slice());
    hash_vector.append(&mut len);

    for txn in input.clone().withdrawal_txn_hashes {
        let mut txn_hash = Vec::from(txn.as_slice());
        hash_vector.append(&mut txn_hash);
    }

    for txn in input.normal_transactions {
        match txn {
            Some(hash) => {
                let mut hash = Vec::from(hash.as_slice());
                hash_vector.append(&mut hash);
            },
            None => {
                continue;
            }
        }
    }

    sp1_zkvm::io::commit_slice(&hash_vector);
}
