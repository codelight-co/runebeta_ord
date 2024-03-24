#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
main() {
    RUST_LOG=info ${SCRIPT_DIR}/ord \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --index-runes --index-transactions \
    --index ${ORD_INDEX_FILE} \
    server --address 0.0.0.0 --http-port ${ORD_PORT} > ${ORD_LOGFILE} 2>&1
}
#For docker
testnet() {
    RUST_LOG=info /usr/local/bin/ord -t \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --index-runes --index-transactions \
    --index /opt/data/runebeta_index_docker.redb \
    server --address 0.0.0.0 --http-port 8088 > /opt/ord.log 2>&1
}

$@