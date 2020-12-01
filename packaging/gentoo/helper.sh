#!/bin/bash
for name in `fgrep name Cargo.lock | cut -d\" -f2`
do
    ver=`fgrep name Cargo.lock -A1 | fgrep $name -A1 | egrep ^version | cut -d\" -f2`
    echo "${name}-${ver}"
done
