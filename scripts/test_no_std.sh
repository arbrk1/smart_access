#!/bin/bash
cargo test --test no_std --no-default-features --features="batch_ct"
cargo test --test no_std --no-default-features --features="batch_ct detach"
