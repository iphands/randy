#!/bin/bash
set -e

export RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native"

if [ "$HOSTNAME" == "handslap" ]
then
    cargo build --release --no-default-features --features timings
    cp ./target/release/randy ./target/release/randy.timings
    cargo build --release --no-default-features
else
    cargo build --release --features timings
    cp ./target/release/randy ./target/release/randy.timings
    cargo build --release
fi

cp ./target/release/randy ./target/release/randy.nostrip
strip -s ./target/release/randy

cp ./target/release/randy.timings ./target/release/randy.timings.nostrip
strip -s ./target/release/randy.timings

ls -lh ./target/release/randy \
   ./target/release/randy.nostrip \
   ./target/release/randy.timings \
   ./target/release/randy.timings.nostrip \
