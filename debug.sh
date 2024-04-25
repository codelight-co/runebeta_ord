#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
#RPC_URL=http://localhost:18332
#RPC_USERNAME=mike
#RPC_PASSWORD=apd3g41pkl
ORD_BITCOIN_RPC_URL=http://localhost:8332
ORD_BITCOIN_RPC_USERNAME=supersats
ORD_BITCOIN_RPC_PASSWORD=de230875c6
FIRST_BLOCK_HEIGHT=840000
# use inside ord code
export ORD_LAST_BLOCK_HEIGHT=850000
export ORD_SUPERSATS_INDEX_ALL_TRANSACTIONS="1"
export RUST_LOG=info
export DATABASE_URL="postgres://postgres:Codelight123@localhost/supersats_mainnet_benchmark"
rm ../supersats_mainnet.redb 
rm ../supersats.log
cargo run -- \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --bitcoin-rpc-username ${ORD_BITCOIN_RPC_USERNAME}  --bitcoin-rpc-password ${ORD_BITCOIN_RPC_PASSWORD} \
    --index-runes --index-transactions \
    --first-inscription-height ${FIRST_BLOCK_HEIGHT} \
    --commit-interval 1 \
    --index ../supersats_mainnet.redb \
    server --address 0.0.0.0 --http-port 8088 > ../supersats.log 2>&1