
use alloy_primitives::{Address, Bytes, U256};
use reth_revm::{
    primitives::{PrecompileOutput, PrecompileResult}, Database,
};

use super::utils::error_wrapper;
use super::utils::get_balances_slot;
use super::utils::get_total_supply_slot;

pub(crate) fn token_balance_update<DB: Database>(
    l2_token: Address,
    to: Address,
    amount: U256,
    mint: bool,
    evmctx: &mut reth_revm::InnerEvmContext<DB>,
) -> PrecompileResult {
    let balances_slot = get_balances_slot(to);
    let total_supply_slot = get_total_supply_slot();

    match evmctx.load_account(l2_token) {
        Ok(_) => {}
        Err(_) => {
            return error_wrapper("AccountLoadError");
        }
    }

    match evmctx.sload(l2_token, balances_slot) {
        Ok(val) => {
            let new_val = if mint {
                amount.checked_add(val.data).expect("Overflow occurred")
            } else {
                amount.checked_sub(val.data).expect("Underflow occurred")
            };

            evmctx.touch(&l2_token);
            match evmctx.sstore(l2_token, balances_slot, new_val) {
                Ok(_) => {
                    // Balances mapping has been updated. Update total supply now
                    match evmctx.sload(l2_token, total_supply_slot) {
                        Ok(total_supply_val) => {
                            let new_total_supply = if mint {
                                total_supply_val
                                    .data
                                    .checked_add(amount)
                                    .expect("Error loading total supply")
                            } else {
                                total_supply_val
                                    .data
                                    .checked_sub(amount)
                                    .expect("Error loading total supply")
                            };

                            match evmctx.sstore(l2_token, total_supply_slot, new_total_supply) {
                                Ok(_) => {
                                    Ok(PrecompileOutput {
                                        gas_used: 0u64,
                                        bytes: Bytes::new(),
                                    })
                                }
                                Err(_) => {
                                    error_wrapper("DBWriteError")
                                }
                            }
                        }
                        Err(_) => {
                            error_wrapper("AccountLoadError")
                        }
                    }
                }
                Err(_) => {
                    error_wrapper("DBWriteError")
                }
            }
        }
        Err(_) => {
            error_wrapper("AccountLoadError")
        }
    }
}

