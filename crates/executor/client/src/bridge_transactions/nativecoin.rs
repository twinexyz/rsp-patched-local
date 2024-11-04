use alloy_primitives::{Address, Bytes, U256};
use reth_revm::{
    primitives::{PrecompileErrors, PrecompileOutput, PrecompileResult},
    Database,
};

use super::utils::error_wrapper;

pub(crate) fn nativecoin_balance_update<DB: Database>(
    to: Address,
    amount: U256,
    mint: bool,
    evmctx: &mut reth_revm::InnerEvmContext<DB>,
) -> PrecompileResult {
    let _ = evmctx
        .load_account(to)
        .map_err(|_| PrecompileErrors::Fatal {
            msg: "Failed to load account".into(),
        })?;
    evmctx.touch(&to);
    match evmctx.load_account(to) {
        Ok(mut acc) => {
            let new_balance = if mint {
                acc.info
                    .balance
                    .checked_add(amount)
                    .ok_or(PrecompileErrors::Fatal {
                        msg: "Overflow".into(),
                    })?
            } else {
                acc.info
                    .balance
                    .checked_sub(amount)
                    .ok_or(PrecompileErrors::Fatal {
                        msg: "Underflow".into(),
                    })?
            };
            acc.info.balance = new_balance;

            PrecompileResult::Ok(PrecompileOutput {
                gas_used: 0,
                bytes: Bytes::new(),
            })
        }
        Err(_) => return error_wrapper("FailedLoadingAccount"),
    }
}
