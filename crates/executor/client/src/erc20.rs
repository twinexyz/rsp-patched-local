use std::sync::Arc;

use alloy_primitives::{keccak256, Address, Bytes, U256};
use alloy_sol_types::sol_data;
use alloy_sol_types::SolType;
use revm::precompile::u64_to_address;
use revm::ContextPrecompile;
use revm::ContextStatefulPrecompile;

// use revm::Database;
use revm::Database;
use revm_primitives::PrecompileError;
use revm_primitives::PrecompileErrors;
use revm_primitives::PrecompileOutput;
use revm_primitives::PrecompileResult;

pub(crate) const ERC20_ADDRESS: u64 = 0x15;

pub const BALANCES_SLOT_POSITION: u8 = 0;
pub const TOTAL_SUPPLY_SLOT_POSITION: u8 = 2;

pub type ContextPrecompileWithAddress<DB> = (Address, ContextPrecompile<DB>);

pub fn erc20_precompiles<DB: Database>() -> impl Iterator<Item = ContextPrecompileWithAddress<DB>> {
    let erc20 = Arc::new(ERC20Token {});
    let precompile: ContextPrecompile<DB> = ContextPrecompile::ContextStateful(erc20);
    let address = u64_to_address(ERC20_ADDRESS);
    [(address, precompile)].into_iter()
}

#[derive(Clone, Debug)]
pub struct ERC20Token {}

impl<DB: Database> ContextStatefulPrecompile<DB> for ERC20Token {
    fn call(
        &self,
        bytes: &alloy_primitives::Bytes,
        gas_limit: u64,
        evmctx: &mut reth_revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let _ = gas_limit;
        // token address, user address, amount
        type TransferParamType = (sol_data::Address, sol_data::Address, sol_data::Uint<256>);

        match TransferParamType::abi_decode(bytes.as_ref(), true) {
            Ok(res) => {
                // load account first, make it warm
                let user_address = reth_primitives::Address::from(*res.1 .0);
                let balances_slot = get_balances_slot(user_address);
                let total_supply_slot = get_total_supply_slot();

                match evmctx.load_account(user_address) {
                    Ok(_) => {}
                    Err(_) => {
                        let err = PrecompileError::Other("AccountLoadError".to_string());
                        return Err(PrecompileErrors::Error(err));
                    }
                }

                match evmctx.sload(user_address, balances_slot) {
                    Ok(val) => {
                        let new_val = res.2.checked_add(val.data).expect("Error loading");
                        evmctx.touch(&user_address);
                        match evmctx.sstore(user_address, balances_slot, new_val) {
                            Ok(_) => {
                                // Balances mapping has been updated. Update total supply now
                                match evmctx.sload(user_address, total_supply_slot) {
                                    Ok(total_supply_val) => {
                                        let new_total_supply = total_supply_val
                                            .data
                                            .checked_add(res.2)
                                            .expect("Error loading total supply");

                                        match evmctx.sstore(
                                            user_address,
                                            total_supply_slot,
                                            new_total_supply,
                                        ) {
                                            Ok(_) => Ok(PrecompileOutput {
                                                gas_used: 0u64,
                                                bytes: Bytes::new(),
                                            }),
                                            Err(_) => {
                                                let err = PrecompileError::Other(
                                                    "DBWriteError".to_string(),
                                                );
                                                Err(PrecompileErrors::Error(err))
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        let err =
                                            PrecompileError::Other("AccountLoadError".to_string());
                                        Err(PrecompileErrors::Error(err))
                                    }
                                }
                            }
                            Err(_) => {
                                let err = PrecompileError::Other("DBWriteError".to_string());
                                Err(PrecompileErrors::Error(err))
                            }
                        }
                    }
                    Err(_) => {
                        let err = PrecompileError::Other("AccountLoadError".to_string());
                        Err(PrecompileErrors::Error(err))
                    }
                }
            }
            Err(_) => {
                let err = PrecompileError::Other("ParseError".to_string());
                Err(PrecompileErrors::Error(err))
            }
        }
    }
}

/// Calculates storage slot for the balances map
/// TODO: Use alloy sol_data as
/// ```rs
/// type SlotKey = (sol_data::Address, sol_data::Uint<256>);
/// let encoded_key = SlotKey::abi_encode(&(address.into(), 0.into()));
/// ```
fn get_balances_slot(address: Address) -> U256 {
    let mut encoded = vec![];

    let addr_bytes = address.to_vec();
    let mut padded_address = vec![0u8; 12];
    padded_address.extend_from_slice(&addr_bytes);
    encoded.extend_from_slice(&padded_address);

    let mut padded_index = vec![0u8; 31];
    padded_index.push(BALANCES_SLOT_POSITION);
    encoded.extend_from_slice(&padded_index);

    let hash = keccak256(encoded);

    U256::from_be_slice(hash.as_ref())
}

/// Returns slot position for total supply in erc20 contract
fn get_total_supply_slot() -> U256 {
    U256::from(TOTAL_SUPPLY_SLOT_POSITION)
}
