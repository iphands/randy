#!/bin/bash
set -e

cargo build --release
cp ./target/release/randy ./target/release/randy.nostrip
strip -s ./target/release/randy
ls -lh ./target/release/randy ./target/release/randy.nostrip
