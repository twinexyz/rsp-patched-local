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

    let mut block_hash = Vec::from(block.hash_slow().as_slice());
    hash_vector.append(&mut block_hash);

    let mut previous_state_root = Vec::from(input.previous_state_root.as_slice());
    hash_vector.append(&mut previous_state_root); 

    let mut state_root = Vec::from(block.state_root.as_slice()); 
    hash_vector.append(&mut state_root);

    let mut txn_root = Vec::from(block.transactions_root.as_slice());
    hash_vector.append(&mut txn_root);

    let mut receipt_root = Vec::from(block.receipts_root.as_slice());
    hash_vector.append(&mut receipt_root);

    let mut chain_id_eth = input.clone().eth_chain_id.to_be_bytes_vec();
    hash_vector.append(&mut chain_id_eth);

    let mut eth_deposit_index = input.clone().deposit_txn_index.to_be_bytes_vec();
    hash_vector.append(&mut eth_deposit_index);

    for txn in input.clone().deposit_txn_hashes {
        let mut deposit_transaction = Vec::from(txn.as_slice());
        hash_vector.append(&mut deposit_transaction);
    }

    let mut eth_withdraw_index = input.clone().withdraw_txn_index.to_be_bytes_vec();
    hash_vector.append(&mut eth_withdraw_index); 

    let mut eth_withdraw_status = input.clone().withdraw_status;
    hash_vector.append(&mut eth_withdraw_index);  

    for txn in input.clone().withdrawal_txn_hashes {
        let mut txn_hash = Vec::from(txn.as_slice());
        hash_vector.append(&mut txn_hash);
    }


    let mut chain_id_solana = input.clone().solana_chain_id.to_be_bytes_vec();
    hash_vector.append(&mut chain_id_solana);

    let mut solana_deposit_index = input.clone().solana_deposit_txn_index.to_be_bytes_vec();
    hash_vector.append(&mut solana_deposit_index);

    for txn in input.clone().solana_deposit_txn_hashes {
        let mut deposit_transaction = Vec::from(txn.as_slice());
        hash_vector.append(&mut deposit_transaction);
    }

    let mut eth_withdraw_index = input.clone().solana_withdraw_txn_index.to_be_bytes_vec();
    hash_vector.append(&mut eth_withdraw_index); 

    let mut eth_withdraw_status = input.clone().solana_withdraw_status;
    hash_vector.append(&mut eth_withdraw_index);  

    for txn in input.clone().solana_withdrawal_txn_hashes {
        let mut txn_hash = Vec::from(txn.as_slice());
        hash_vector.append(&mut txn_hash);
    }


    for txn in input.clone().dvn_transactions {
        let mut tx_hash = Vec::from(txn.as_slice());
        hash_vector.append(&mut tx_hash);
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
