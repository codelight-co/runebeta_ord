#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
BITCOIN_RPC_URL_TESTNET=http://192.168.1.253:18332
BITCOIN_RPC_URL_MAINNET=http://192.168.1.253:8332
main() {
    RUST_LOG=info ${SCRIPT_DIR}/ord \
    --bitcoin-rpc-url http://192.168.1.253:8332 \
    --index-runes --index-transactions \
    --index /mnt/ordinals/codelight/bitcoin/ord/data/0.16.0/runebeta_index.redb \
    server --address 0.0.0.0 --http-port 8090 > runebeta_ord.log 2>&1
}
#For docker
testnet() {
    RUST_LOG=info /usr/local/bin/ord -t \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --index-runes --index-transactions \
    --index /opt/data/runebeta_index.redb \
    server --address 0.0.0.0 --http-port 8088 > /opt/ord.log 2>&1
}

debug() {
    RUST_LOG=debug cargo run -- -t --bitcoin-rpc-password bitcoincodelight \
    --bitcoin-rpc-url http://192.168.1.254:18332 \
    --bitcoin-rpc-username bitcointestnet \
    --index-runes --index-transactions \
    --index /Users/viettai/workspace/bitcoin/runebeta_ord/index.redb \
    server --address 0.0.0.0 --http-port 8088
}

tunnel() {
    ssh -L 8090:192.168.1.253:8090 scalar
}
$@