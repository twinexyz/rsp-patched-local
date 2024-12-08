use alloy_sol_types::sol;

pub(crate) const ETHEREUM_MAINNET_CHAINID: u64 = 1;
pub(crate) const ETHEREUM_SEPOLIA_CHAINID: u64 = 11155111;
pub(crate) const ETHEREUM_HOLESKY_CHAINID: u64 = 17000;
pub(crate) const SOLANA_CHAINID: u64 = 900;

sol!(
    event Deposit(uint256 chainId, uint256 index, bytes32 combinedHash);
);

sol!(
    event WithDraw(uint256 chainId, uint256 index, bytes statusBytes,bytes32 combinedHash);
);
