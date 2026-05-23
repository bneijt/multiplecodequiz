#!/bin/bash
set -eo pipefail
export RUST_BACKTRACE=1
if [[ ! -e quiz_data.json ]]; then
    cargo run -p preprocess -- --repo crates
fi
cd crates/frontend && trunk serve
