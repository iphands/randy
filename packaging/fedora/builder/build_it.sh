#!/bin/bash

cd /home/iphands/build
cp /randy/packaging/fedora/randy.spec .
# spectool -g ./randy.spec

time fedpkg --release f34 local
time rpmbuild --rebuild randy*.src.rpm

