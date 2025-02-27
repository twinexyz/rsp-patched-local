use alloy_provider::ReqwestProvider;
use clap::Parser;
use execute::process_execution_report;
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, CHAIN_ID_DEVNET, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET, CHAIN_ID_SEPOLIA,
};
use rsp_host_executor::HostExecutor;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use tracing_subscriber::{
    filter::EnvFilter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

mod execute;

mod cli;
use cli::ProviderArgs;

mod eth_proofs;

/// The arguments for the host executable.
#[derive(Debug, Clone, Parser)]
struct HostArgs {
    /// The block number of the block to execute.
    #[clap(long)]
    block_number: u64,
    #[clap(long)]
    to_block: Option<u64>,
    #[clap(flatten)]
    provider: ProviderArgs,

    /// The path to the genesis json file to use for the execution.
    #[clap(long)]
    genesis_path: Option<PathBuf>,

    /// generate a proof 
    #[clap(long)]
    prove: bool,

    /// generate a dummy proof by just executing the program 
    #[clap(long)]
    execute: bool,

    /// Optional path to the directory containing cached client input. A new cache file will be
    /// created from RPC data if it doesn't already exist.
    #[clap(long)]
    cache_dir: Option<PathBuf>,

    /// The path to the CSV file containing the execution data.
    #[clap(long, default_value = "report.csv")]
    report_path: PathBuf,

    /// Optional ETH proofs endpoint.
    #[clap(long, env, requires("eth_proofs_api_token"))]
    eth_proofs_endpoint: Option<String>,

    /// Optional ETH proofs API token.
    #[clap(long, env)]
    eth_proofs_api_token: Option<String>,

    /// Optional ETH proofs cluster ID.
    #[clap(long, default_value_t = 1)]
    eth_proofs_cluster_id: u64,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize the environment variables.
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize the logger.
    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from_default_env()).init();

    // Parse the command line arguments.
    let args = HostArgs::parse();
    let provider_config = args.provider.into_provider().await?;

    let variant = match &args.genesis_path {
        Some(genesis_path) => ChainVariant::from_genesis_path(genesis_path)?,
        None => match provider_config.chain_id {
            CHAIN_ID_ETH_MAINNET => ChainVariant::mainnet(),
            CHAIN_ID_OP_MAINNET => ChainVariant::op_mainnet(),
            CHAIN_ID_LINEA_MAINNET => ChainVariant::linea_mainnet(),
            CHAIN_ID_SEPOLIA => ChainVariant::sepolia(),
            CHAIN_ID_DEVNET => ChainVariant::devnet(),
            _ => {
                eyre::bail!("Unknown chain ID: {}", provider_config.chain_id);
            }
        },
    };

    if args.genesis_path.is_some() && variant.chain_id() != provider_config.chain_id {
        eyre::bail!("The chain ID in the genesis file does not match the provided RPC");
    }

    let client_input_from_cache = try_load_input_from_cache(
        args.cache_dir.as_ref(),
        provider_config.chain_id,
        args.block_number,
    )?;

    let (client_input, to_block) = match (client_input_from_cache, provider_config.rpc_url) {
        (Some(client_input_from_cache), _) => {
            (vec![client_input_from_cache], args.to_block.unwrap())
        }
        (None, Some(rpc_url)) => {
            // Cache not found but we have RPC
            // Setup the provider.
            let provider = ReqwestProvider::new_http(rpc_url);

            // Setup the host executor.
            let host_executor = HostExecutor::new(provider);

            let mut client_input = Vec::new();
            let to_block = match args.to_block {
                Some(to_block) => to_block,
                None => args.block_number,
            };

            let blocks = host_executor
                .get_desired_blocks(args.block_number, to_block)
                .await
                .expect("failed to get desired blocks from RPC");

            for i in 0..blocks.len() - 1 {
                let cl_input = host_executor
                    .execute(blocks[i].clone(), blocks[i + 1].clone(), variant.clone())
                    .await
                    .expect("failed to execute host");
                client_input.push(cl_input);
            }

            if let Some(ref cache_dir) = args.cache_dir {
                let input_folder = cache_dir.join(format!("input/{}", provider_config.chain_id));
                if !input_folder.exists() {
                    std::fs::create_dir_all(&input_folder)?;
                }

                let input_path = input_folder.join(format!("{}.bin", args.block_number));
                let mut cache_file = std::fs::File::create(input_path)?;

                bincode::serialize_into(&mut cache_file, &client_input)?;
            }

            (client_input, to_block)
        }
        (None, None) => {
            eyre::bail!("cache not found and RPC URL not provided")
        }
    };

    // Generate the proof.
    let client = ProverClient::from_env();

    // Setup the proving key and verification key.
    let (pk, vk) = client.setup(match variant {
        ChainVariant::Ethereum(_) => include_elf!("rsp-client-eth"),
        ChainVariant::Optimism(_) => include_elf!("rsp-client-op"),
        ChainVariant::Linea(_) => include_elf!("rsp-client-linea"),
        ChainVariant::Devnet(_) => include_elf!("rsp-client-local"),
    });

    // Execute the block inside the zkVM.
    let mut stdin = SP1Stdin::new();
    let buffer = bincode::serialize(&client_input).unwrap();
    stdin.write_vec(buffer);

    // Only execute the program.
    let (output, execution_report) = client.execute(&pk.elf, &stdin).run().unwrap();

    process_execution_report(variant, client_input, execution_report, args.report_path.clone())?;

    let proof_dir = "proofs";
    if let Ok(exists) = fs::exists(proof_dir) {
        if !exists {
            fs::create_dir(proof_dir).unwrap();
        }
    }

    if args.prove {
        println!("Starting proof generation.");
        let proof = client.prove(&pk, &stdin).groth16().run().expect("Proving should work.");

        let proof_json = serde_json::to_string(&proof).expect("could not serialized the proof");
        save_proof_to_file(proof_json, proof_dir.to_string(), args.block_number, to_block);

        client.verify(&proof, &vk).expect("proof verification should succeed");
    } else if args.execute {
        let public_value: String = output.raw();
        let proof_json =
            serde_json::to_string(&public_value).expect("couldnot serialize the proof");
        save_proof_to_file(proof_json, proof_dir.to_string(), args.block_number, to_block);
    } else {
        panic!("should run in proving or executing mode");
    }

    Ok(())
}

fn save_proof_to_file(proof_json: String, proof_dir: String, start_block: u64, end_block: u64) {
    let file_name = format!("{}/execution_proof_{}_{}.proof", proof_dir, start_block, end_block);
    let mut proof_file = File::create(&file_name).expect("file creation error");
    proof_file.write_all(proof_json.as_bytes()).expect("error writing proof to the file");
}

fn try_load_input_from_cache(
    cache_dir: Option<&PathBuf>,
    chain_id: u64,
    block_number: u64,
) -> eyre::Result<Option<ClientExecutorInput>> {
    Ok(if let Some(cache_dir) = cache_dir {
        let cache_path = cache_dir.join(format!("input/{}/{}.bin", chain_id, block_number));

        if cache_path.exists() {
            // TODO: prune the cache if invalid instead
            let mut cache_file = std::fs::File::open(cache_path)?;
            let client_input: ClientExecutorInput = bincode::deserialize_from(&mut cache_file)?;

            Some(client_input)
        } else {
            None
        }
    } else {
        None
    })
}
