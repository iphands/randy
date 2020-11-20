#!/bin/bash
set -e

RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native" cargo build --release
cp ./target/release/randy ./target/release/randy.nostrip
strip -s ./target/release/randy
ls -lh ./target/release/randy ./target/release/randy.nostrip
