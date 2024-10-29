#!/bin/sh
start=$(date +%s) 
rsp --block-number $1 --rpc-url http://localhost:8545/ --chain-id 1337 --prove
end=$(date +%s) 
elapsed=$(( end - start ))

echo "Total time: $elapsed seconds"