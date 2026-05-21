#!/bin/bash
set -eo pipefail
if [[ ! -e crates/frontend/public/quiz_data.json ]]; then
    cargo run -p preprocess -- --repo crates
fi
cd crates/frontend && trunk serve
