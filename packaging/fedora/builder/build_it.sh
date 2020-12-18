#!/bin/bash

cd /home/iphands/build
cp /randy/packaging/fedora/randy.spec .

if [ "$DO_LOCAL" == "false" ]
then
    spectool -g ./randy.spec
fi

time fedpkg --release f34 local
time rpmbuild --rebuild randy*.src.rpm

