#!/bin/bash
set -e
rm -rf ./working/*
cargo package --allow-dirty
cp  ../../../target/package/randy*crate ./working/
docker build . -t fedora_randy_builder
docker run --rm -ti \
       -v /home/iphands/prog/rust/ronky:/randy \
       -v /home/iphands/prog/rust/ronky/packaging/fedora/builder/working:/home/iphands/build \
       -v /home/iphands/prog/rust/ronky/packaging/fedora/builder/output:/home/iphands/rpmbuild \
       fedora_randy_builder
