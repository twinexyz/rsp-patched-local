## To prove local testnet geth blocks 

## TLDR
All the following commands below can be executed by running `run.sh` in the root directory. Read the following steps for manual proving.


Run the geth server in dev mode, with http enabled. The default port is at http://127.0.0.1:8545
```sh
geth --dev --http --http.api eth,web3,net --http.corsdomain "https://remix.ethereum.org/" 
```

On another terminal, open up the geth console, and get the faucet account.
```sh
geth attach http://127.0.0.1:8545
# in the console
> eth.accounts[0]
 ["0xabcd....1234"]
```

Copy this address. 

Quit the console with `ctrl + d`

Then, enter the following command 

```sh
geth --dev dumpgenesis | jq -r . > genesis.json
```

This genesis file does not contain the `faucet` account, and `HistoryStorageAddress` contract.

Add the following to the alloc of the genesis as:
```json
{
	"alloc": {
		...,
		{
	    "abcd....1234": {
	      "balance": "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7"
	    },
	    "0aae40965e6800cd9b1f4b05ff21581047e3f91e": {
	      "code": "0x3373fffffffffffffffffffffffffffffffffffffffe1460575767ffffffffffffffff5f3511605357600143035f3511604b575f35612000014311604b57611fff5f3516545f5260205ff35b5f5f5260205ff35b5f5ffd5b5f35611fff60014303165500",
	      "balance": "0x0",
	      "nonce": "0x1"
	    }
	}
}
```


```sh
cp genesis.json $PROJECT_ROOT/crates/primitives/res/genesis/genesis.json
```

> Note: If you restart geth --dev server, new faucet account will be created. So, you'll need to update the genesis file again.

Now, we need to generate blocks to prove using rsp. To create the blocks, enter the geth console again.
```sh
geth attach http://127.0.0.1:8545
```

The following command sends 50 eth to random address. This transaction will create a block. Enter this command a few times.
```sh
eth.sendTransaction({from: eth.accounts[0], to: "0xd19de419e6d0ca1907c09b9ee25a071d007a80b4", value: web3.toWei(50, "ether")})
```

Now, there are blocks on our local geth environment, 
To verify the blocks, run the following command
```sh
cargo run --bin rsp --release -- --block-number 4 --rpc-url http://127.0.0.1:8545 --chain-id 1337
```
we can prove them using rsp as 

```sh
SP1_PROVER=""
SP1_PRIVATE_KEY=""
# GPU proving
cargo run --bin rsp --release --features cuda -- --block-number 4 --rpc-url http://127.0.0.1:8545 --chain-id 1337 --prove
```

```sh
# CPU Proving
cargo run --bin rsp --release --features cuda -- --block-number 4 --rpc-url http://127.0.0.1:8545 --chain-id 1337 --prove
```


