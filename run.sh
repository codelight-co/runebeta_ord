#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
RUST_LOG=info ${SCRIPT_DIR}/ord -t --bitcoin-rpc-password bitcoincodelight \
    --bitcoin-rpc-url https://192.168.1.254:18332 \
    --bitcoin-rpc-username bitcointestnet \
    --index-runes --index-transactions \
    --index ./index.redb \
    server --address 0.0.0.0 --http-port 8088 > ord.log 2>&1
