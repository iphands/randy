#!/bin/bash
set -e

if [ "$HOSTNAME" == "handslap" ]
then
    RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native" cargo build --release --no-default-features
else
    RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native" cargo build --release --features timings
    cp ./target/release/randy ./target/release/randy.timings
    RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native" cargo build --release
fi

cp ./target/release/randy ./target/release/randy.nostrip
strip -s ./target/release/randy
ls -lh ./target/release/randy ./target/release/randy.nostrip
