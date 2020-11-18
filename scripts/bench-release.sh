#!/bin/bash
set -e

cargo build --release --no-default-features --features "benchmark"
cp ./target/release/randy ./target/release/randy.nostrip
strip -s ./target/release/randy
ls -lh ./target/release/randy ./target/release/randy.nostrip
./target/release/randy $1
