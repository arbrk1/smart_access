#!/bin/bash
cargo test
cargo test --no-default-features --features "collections batch_ct batch_rt"
cargo test --no-default-features --features "collections detach"
cargo test --no-default-features --features "collections"
