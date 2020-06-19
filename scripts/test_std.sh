#!/bin/bash
cargo test --no-default-features --features "std_collections"
cargo test --no-default-features --features "std_collections batch_ct batch_rt"
cargo test --no-default-features --features "std_collections detach"
