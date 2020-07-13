#!/bin/bash
cargo test
cargo test --features "std_hashmap"
cargo test --no-default-features --features "collections hashbrown batch_ct batch_rt"
cargo test --no-default-features --features "collections hashbrown detach"
cargo test --no-default-features --features "collections hashbrown"
