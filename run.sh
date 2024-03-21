#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"

testnet() {
    RUST_LOG=info ${SCRIPT_DIR}/ord -t --bitcoin-rpc-password bitcoincodelight \
    --bitcoin-rpc-url http://192.168.1.254:18332 \
    --bitcoin-rpc-username bitcointestnet \
    --index-runes --index-transactions \
    --index ./index.redb \
    server --address 0.0.0.0 --http-port 8088 > ord.log 2>&1
}

debug() {
    RUST_LOG=debug cargo run -- -t --bitcoin-rpc-password bitcoincodelight \
    --bitcoin-rpc-url http://192.168.1.254:18332 \
    --bitcoin-rpc-username bitcointestnet \
    --index-runes --index-transactions \
    --index /Users/viettai/workspace/bitcoin/runebeta_ord/index.redb \
    server --address 0.0.0.0 --http-port 8088
}

$@