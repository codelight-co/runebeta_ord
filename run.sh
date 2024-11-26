#!/usr/bin/env bash

curl --user scalar:scalartestnet4 --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "getblockhash", "params": [0]}' -H 'content-type: text/plain;' http://testnet4.btc.scalar.org
RUST_LOG=trace cargo run --bin ord -- --config ord-scalar.yaml server
