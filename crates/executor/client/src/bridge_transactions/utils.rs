use alloy_primitives::{keccak256, Address, U256};
use reth_revm::primitives::{PrecompileError, PrecompileErrors, PrecompileResult};

pub(crate) const BALANCES_SLOT_POSITION: u8 = 0;
pub(crate) const RECEIPT_SLOT_POSITION: u8 = 0;
pub(crate) const TOTAL_SUPPLY_SLOT_POSITION: u8 = 2;

pub(crate) fn error_wrapper(y: &str) -> PrecompileResult {
    let err = PrecompileError::Other(y.to_string());
    Err(PrecompileErrors::Error(err))
}

/// Calculates storage slot for the balances map
/// TODO: Use alloy sol_data as
/// ```rs
/// type SlotKey = (sol_data::Address, sol_data::Uint<256>);
/// let encoded_key = SlotKey::abi_encode(&(address.into(), 0.into()));
/// ```
pub(crate) fn get_balances_slot(address: Address) -> U256 {
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
pub(crate) fn get_total_supply_slot() -> U256 {
    U256::from(TOTAL_SUPPLY_SLOT_POSITION)
}

/// Returns slot position for receipt map
pub(crate) fn get_receipt_root_slot(height: U256) -> U256 {
    let mut encoded = vec![];

    encoded.extend_from_slice(&height.to_be_bytes_vec());

    let mut padded_index = vec![0u8; 31];
    padded_index.push(RECEIPT_SLOT_POSITION);
    encoded.extend_from_slice(&padded_index);

    let hash = keccak256(encoded);

    U256::from_be_slice(hash.as_ref())
}
