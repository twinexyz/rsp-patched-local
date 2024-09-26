#!/bin/bash

# Step 1: Start geth in the background
geth --dev --http --http.api eth,web3,net --http.corsdomain "https://remix.ethereum.org/" &
geth_pid=$!

# Check if geth started successfully
if [ $? -ne 0 ]; then
    echo "Failed to start geth."
    exit 1
fi

# Step 2: Wait for geth to fully start
sleep 10  # Adjust this if needed to ensure geth is up and running

# Step 3: Dump the current genesis
geth --dev dumpgenesis | jq -r . > genesis.json
if [ $? -ne 0 ]; then
    echo "Failed to dump genesis using geth."
    kill $geth_pid  # Stop geth if dumping failed
    exit 1
fi

# Step 4: Get the first account from geth
account=$(geth attach --exec "eth.accounts[0]" http://127.0.0.1:8545)
if [ $? -ne 0 ]; then
    echo "Failed to connect to geth or fetch the account."
    kill $geth_pid  # Stop geth if fetching account failed
    exit 1
fi

if [ -z "$account" ]; then
    echo "No accounts found or failed to retrieve the account."
    kill $geth_pid  # Stop geth if account is empty
    exit 1
fi

# Step 5: Remove the "0x" prefix from the account
cleaned=${account//\"/}
trimmed_account=${cleaned#0x} 

# Define the output_balance and history_storage entries as JSON objects
output_balance=$(jq -n --arg key "$trimmed_account" \
    '{($key): { "balance": "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7" }}')

history_storage=$(jq -n '{
  "0aae40965e6800cd9b1f4b05ff21581047e3f91e": {
    "code": "3373fffffffffffffffffffffffffffffffffffffffe1460575767ffffffffffffffff5f3511605357600143035f3511604b575f35612000014311604b57611fff5f3516545f5260205ff35b5f5f5260205ff35b5f5ffd5b5f35611fff60014303165500",
    "balance": "0",
    "nonce": "1"
  }
}')

# Step 6: Merge the output_balance and history_storage into the alloc field of genesis.json
jq --argjson output_balance "$output_balance" \
   --argjson history_storage "$history_storage" \
   '.alloc += ($output_balance + $history_storage)' genesis.json > genesis_updated.json

if [ $? -ne 0 ]; then
    echo "Failed to update genesis.json using jq."
    kill $geth_pid  # Stop geth if update failed
    exit 1
fi

# Replace the original genesis.json with the updated one
mv genesis_updated.json crates/primitives/res/genesis/genesis.json
rm -r genesis.json

# Stop geth
kill $geth_pid

echo "Added the account and history storage to genesis.json successfully."

echo "Do a few transactions to simulate blocks"

# Creates 10 transactions from faucet wallet to a random address, creates 10 blocks
for i in {1..10}
do 
    geth attach --exec "eth.sendTransaction({from: eth.accounts[0], to: '0xd19de419e6d0ca1907c09b9ee25a071d007a80b4', value: web3.toWei(50, 'ether')})" http://127.0.0.1:8545
    sleep 2
done

echo "Build the program.."
cargo install --locked --path bin/host

# Check if $1 is a number, if yes, try to prove that block, else default to block number 4
if [[ $1 =~ ^[0-9]+$ ]]; then
    value=$1
else
    value=4
fi

echo "Proving block " $value
cargo run --bin rsp --release -- --block-number $value --rpc-url http://localhost:8545/ --chain-id 1337 --prove
