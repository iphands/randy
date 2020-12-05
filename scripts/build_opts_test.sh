#!/bin/bash

mkdir -p ./stash/test_bins/

# FEATURES=(avx avx2 sse sse2 sse3 sse4a ssse3)
# for feature in ${FEATURES[@]}
# do
#     echo "TARGET FEATURE: $feature"
#     FULL="+${feature}"
#     for off in ${FEATURES[@]}
#     do
#         if [ `[ "$off" != "$feature" ] ; echo $?` == 0 ]
#         then
#             FULL="$FULL,-${off}"
#         fi
#     done
#     echo "DOING $FULL"
#     export RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native -C target-feature=$FULL"
#     cargo build --release --features timings,nvidia,sensors
#     strip -s ./target/release/randy
#     mv ./target/release/randy ./stash/test_bins/randy_${feature}
# done


FEATURES=(avx avx2 sse3 sse4a ssse3)
for feature in ${FEATURES[@]}
do
    echo "TARGET FEATURE: $feature"
    FULL="+sse,+sse2,+${feature}"

    for off in ${FEATURES[@]}
    do
        if [ `[ "$off" != "$feature" ] ; echo $?` == 0 ]
        then
            FULL="$FULL,-${off}"
        fi
    done

    echo "DOING $FULL"
    export RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C target-cpu=native -C target-feature=$FULL"
    cargo build --release --features timings,sensors
    strip -s ./target/release/randy
    mv ./target/release/randy ./stash/test_bins/randy_${feature}
done
