use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use reth_optimism_chainspec::OP_MAINNET;

/// Returns the [ChainSpec] for Ethereum mainnet.
pub fn mainnet() -> eyre::Result<ChainSpec> {
    Ok(ChainSpecBuilder::mainnet().shanghai_activated().build())
}

pub fn devnet() -> eyre::Result<ChainSpec> {
    println!("crates/primitives/src/chain_spec.rs:: this should have called the genesis block.");
    let genesis = include_str!("../res/genesis/genesis.json");
    let genesis: reth_primitives::Genesis = serde_json::from_str(genesis).unwrap();
    println!("crates/primitives/src/chain_spec.rs:: no issues in loading the genesis block.");
    Ok(ChainSpecBuilder::default()
        .chain(Chain::dev())
        .genesis(genesis)
        .cancun_activated()
        .build())
}

/// Returns the [ChainSpec] for OP Mainnet.
pub fn op_mainnet() -> eyre::Result<ChainSpec> {
    Ok((*OP_MAINNET.clone()).clone())
}

/// Returns the [ChainSpec] for Linea Mainnet.
pub fn linea_mainnet() -> eyre::Result<ChainSpec> {
    let genesis = include_str!("../res/genesis/linea-mainnet.json");
    let genesis: reth_primitives::Genesis = serde_json::from_str(genesis).unwrap();

    // note: Linea has London activated; but setting Paris tricks reth into disabling
    // block rewards, which we need for Linea (clique consensus) to work
    let chain_spec = ChainSpecBuilder::default()
        .chain(Chain::linea())
        .genesis(genesis)
        .paris_activated()
        .build();

    Ok(chain_spec)
}
