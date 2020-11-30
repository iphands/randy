#!/bin/bash

rm -rf /home/iphands/flatpaktest/.flatpak-builder/* build-dir/

set -e

pushd ../../
cargo build
popd

flatpak-builder --install --user build-dir org.ahands.ian.Randy.yml
flatpak-builder --run ./build-dir org.ahands.ian.Randy.yml randy
