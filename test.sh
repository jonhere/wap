#!/bin/bash -e
reset
cargo build --release --examples
node -e node -e 'require("./target/wasm32-unknown-unknown/release/wap.js").wap("target/wasm32-unknown-unknown/release/examples/test_node.wasm")'

