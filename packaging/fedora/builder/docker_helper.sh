#!/bin/bash
ROOT=`git rev-parse --show-toplevel`

set -e
rm -rf ./working/*

DO_LOCAL=false
if [ "local" == "$1" ]
then
    cargo package --allow-dirty
    cp  ../../../target/package/randy*crate ./working/
    DO_LOCAL=true
fi

docker build . -t fedora_randy_builder
docker run --rm -ti \
       -e DO_LOCAL=$DO_LOCAL \
       -v ${ROOT}:/randy:ro \
       -v ${ROOT}/packaging/fedora/builder/working:/home/iphands/build:rw \
       -v ${ROOT}/packaging/fedora/builder/output:/home/iphands/rpmbuild:rw \
       fedora_randy_builder
