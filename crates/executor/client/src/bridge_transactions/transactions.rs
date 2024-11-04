use std::sync::Arc;

// use alloy::sol;
use alloy_primitives::{Address, Bytes};
use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::SolEvent;
use alloy_sol_types::SolType;
use alloy_sol_types::{sol, sol_data};
use reth_primitives::ReceiptWithBloom;
use reth_revm::{
    precompile::u64_to_address, primitives::PrecompileResult, ContextPrecompile,
    ContextStatefulPrecompile, Database,
};
use revm_primitives::hex::FromHex;

use super::nativecoin::nativecoin_balance_update;
use super::token::token_balance_update;
use super::utils::error_wrapper;
use crate::merkle_verifier::{MerkleVerifier, MerkleVerifierError};

pub(crate) const MESSENGER_CONTRACT_ADDRESS: u64 = 0x18;
pub(crate) const TRANSACTIONS_ADDRESS: u64 = 0x15;

use super::utils::get_receipt_root_slot;

pub(crate) type ContextPrecompileWithAddress<DB> = (Address, ContextPrecompile<DB>);

pub(crate) fn bridge_precompiles<DB: Database>(
) -> impl Iterator<Item = ContextPrecompileWithAddress<DB>> {
    let txns = Arc::new(Transactions {});
    let precompile: ContextPrecompile<DB> = ContextPrecompile::ContextStateful(txns);
    let address = u64_to_address(TRANSACTIONS_ADDRESS);
    [(address, precompile)].into_iter()
}

#[derive(Clone)]
pub(crate) struct Transactions {}

sol! {
    #[derive(Debug)]
    event DepositERC20(
        address indexed l1Token,
        address indexed l2Token,
        address indexed from,
        address to,
        uint256 amount,
        bytes data,
        uint256 blockHeight
    );

    #[derive(Debug)]
    event FinalizeWithdrawERC20(
        address indexed l1Token,
        address indexed l2Token,
        address indexed from,
        address to,
        uint256 amount,
        bytes data,
        uint256 blockHeight
    );

    #[derive(Debug)]
    event DepositETH(address indexed from, address indexed to, uint256 amount, uint256 blockHeight);

    #[derive(Debug)]
    event FinalizeWithdrawETH(address indexed from, address indexed to, uint256 amount, uint256 blockHeight);
}

impl<DB: Database> ContextStatefulPrecompile<DB> for Transactions {
    fn call(
        &self,
        bytes: &alloy_primitives::Bytes,
        _gas_limit: u64,
        evmctx: &mut reth_revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        // only messenger contract allowed to call this
        // TODO: Uncomment this:
        // let msg_sender = evmctx.env.tx.caller;
        // if !msg_sender.eq(&u64_to_address(MESSENGER_CONTRACT_ADDRESS)) {
        //     return error_wrapper("invalid sender");
        // }

        type MerkleParamType = (sol_data::Array<sol_data::Bytes>, sol_data::Array<sol_data::Bytes>);
        match MerkleParamType::abi_decode_sequence(bytes.as_ref(), true) {
            Ok((txns, proof)) => {
                if txns.len() != proof.len() {
                    return error_wrapper("InvalidSize");
                }
                for i in 0..txns.len() {
                    let receipt = alloy_rlp::decode_exact::<ReceiptWithBloom>(&txns[i]).unwrap();

                    let transfer_logs = receipt.receipt.logs;
                    for log in transfer_logs {
                        let topic0 = log.topics().first();
                        match topic0 {
                            Some(&DepositERC20::SIGNATURE_HASH) => {
                                let dep = match DepositERC20::decode_log(&log, true) {
                                    Ok(res) => res,
                                    Err(e) => return error_wrapper(&format!("{e:?}")),
                                };

                                if let Err(e) = verify_merkle_proof(
                                    txns[i].clone(),
                                    proof[i].clone(),
                                    dep.blockHeight,
                                    evmctx,
                                ) {
                                    return error_wrapper(&format!("{e:?}"));
                                }

                                return token_balance_update(
                                    dep.l2Token,
                                    dep.to,
                                    dep.amount,
                                    true,
                                    evmctx,
                                );
                            }
                            Some(&FinalizeWithdrawERC20::SIGNATURE_HASH) => {
                                let dep = match FinalizeWithdrawERC20::decode_log(&log, true) {
                                    Ok(res) => res,
                                    Err(e) => return error_wrapper(&format!("{e:?}")),
                                };
                                if let Err(e) = verify_merkle_proof(
                                    txns[i].clone(),
                                    proof[i].clone(),
                                    dep.blockHeight,
                                    evmctx,
                                ) {
                                    return error_wrapper(&format!("{e:?}"));
                                }

                                return token_balance_update(
                                    dep.l2Token,
                                    dep.to,
                                    dep.amount,
                                    false,
                                    evmctx,
                                );
                            }
                            Some(&DepositETH::SIGNATURE_HASH) => {
                                let dep = match DepositETH::decode_log(&log, true) {
                                    Ok(res) => res,
                                    Err(e) => return error_wrapper(&format!("{e:?}")),
                                };
                                if let Err(e) = verify_merkle_proof(
                                    txns[i].clone(),
                                    proof[i].clone(),
                                    dep.blockHeight,
                                    evmctx,
                                ) {
                                    return error_wrapper(&format!("{e:?}"));
                                }
                                return nativecoin_balance_update(dep.to, dep.amount, true, evmctx);
                            }
                            Some(&FinalizeWithdrawETH::SIGNATURE_HASH) => {
                                let dep = match FinalizeWithdrawETH::decode_log(&log, true) {
                                    Ok(res) => res,
                                    Err(e) => return error_wrapper(&format!("{e:?}")),
                                };
                                if let Err(e) = verify_merkle_proof(
                                    txns[i].clone(),
                                    proof[i].clone(),
                                    dep.blockHeight,
                                    evmctx,
                                ) {
                                    return error_wrapper(&format!("{e:?}"));
                                }
                                return nativecoin_balance_update(
                                    dep.to, dep.amount, false, evmctx,
                                );
                            }
                            _ => {
                                // todo: remove this and return error
                                let to =
                                    Address::from_hex("1111111111111111111111111111111111111111")
                                        .unwrap();
                                let amount = U256::from(100u64);

                                return nativecoin_balance_update(to, amount, true, evmctx);
                                // return error_wrapper("InvalidTransaction");
                            }
                        }
                    }
                }
            }
            Err(_) => return error_wrapper("DecodeError"),
        }

        return error_wrapper("FailedParsing");
    }
}

/// verify this block height has been consensus verified and fetch the receipt root
/// merkle prove the receipt against the merkle root
/// proof_data should contain abi encoded nibble and proof for a transaction
fn verify_merkle_proof<DB: Database>(
    txn_data: Bytes,
    proof_data: Bytes,
    height: U256,
    evmctx: &mut reth_revm::InnerEvmContext<DB>,
) -> Result<bool, MerkleVerifierError> {
    // fetch using evmctx and height from precompile or messenger contract
    // let receipt_root_slot = get_receipt_root_slot(height);
    // let messenger_contract = u64_to_address(MESSENGER_CONTRACT_ADDRESS);
    // let receipt_root: FixedBytes<32>;

    // match evmctx.load_account(u64_to_address(MESSENGER_CONTRACT_ADDRESS)) {
    //     Ok(_) => match evmctx.sload(messenger_contract, receipt_root_slot) {
    //         Ok(root) => {
    //             receipt_root = FixedBytes::from(root.data);
    //         }
    //         Err(_) => return Err(MerkleVerifierError::FailedToLoad),
    //     },
    //     Err(_) => return Err(MerkleVerifierError::FailedToLoad),
    // }
    
    
    // TODO: comment the following line, and uncomment above

    let receipt_root: FixedBytes<32> =
        FixedBytes::from_hex("9c5925245db5d0a87e235b3e582d2fe6407b61ba16d88bb68614b016f5ec4b64")
            .unwrap();

    match MerkleVerifier::verify_proof(txn_data, receipt_root, proof_data) {
        Ok(_) => Ok(true),
        Err(e) => return Err(e.into()),
    }
}
