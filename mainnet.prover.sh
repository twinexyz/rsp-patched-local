#!/bin/sh
start=$(date +%s) 
rsp --block-number $1 --rpc-url https://eth-mainnet.g.alchemy.com/v2/GLYjGj3YzX6ayV_RlGe--fhdxbcuyDBv  --chain-id 1  --prove
end=$(date +%s) 
elapsed=$(( end - start ))

echo "Total time: $elapsed seconds"
