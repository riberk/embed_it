#! /bin/bash
set -e

compile() (
    local FEATURE=$1
    export TIMEFORMAT=%R
    BUILD_TIME=$( { time cargo build --tests --quiet --features=$FEATURE --timings; } 2>/dev/null)
    ENTRIE_COUNT=$(cargo test --features=$FEATURE --quiet -- --nocapture | grep '###ENTRIES_COUNT###:' | sed -E "s/###ENTRIES_COUNT###: *([0123456789]+)/\1/g")
    echo "time: ${BUILD_TIME} sec; entries_count: ${ENTRIE_COUNT}; feature: ${FEATURE} "

)

test() (
    cd ./compilation_bench
    
    echo "Cleaning target..."
    cargo clean --quiet
    
    # build without features to build dependencies
    echo "Building deps..."
    export TIMEFORMAT=%R
    TIME=$( { time cargo build --quiet; } 2>/dev/null)
    echo "Deps built in ${TIME} sec"

    compile "bench-files"
    compile "bench-dirs"
    compile "bench-nested-dirs"
    compile "bench-nesting"

    compile "bench-lots-of-files"
    compile "bench-lots-of-dirs"
    compile "bench-lots-of-nesting-items"
)

test
