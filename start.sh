#!/bin/bash
cargo run -p preprocess -- crates
cd crates/frontend && trunk serve
