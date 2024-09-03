#!/bin/sh
cargo install --locked --path bin/host
cargo run --bin rsp --release -- --block-number $1 --rpc-url http://localhost:8545/ --chain-id 1337 --prove
