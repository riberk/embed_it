#!/bin/bash
set -e

cargo +nightly llvm-cov clean --workspace
cargo +nightly llvm-cov nextest --all-features      --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --all-features      --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest                     --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov                             --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="md5"    --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="md5"    --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="sha1"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="sha1"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="sha2"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="sha2"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="sha3"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="sha3"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="blake3" --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="blake3" --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="gzip"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="gzip"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="zstd"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="zstd"   --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --features="brotli" --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs"       --no-report
cargo +nightly llvm-cov         --features="brotli" --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo llvm-cov report --ignore-filename-regex=".+/macros/src/test_helpers.rs" --lcov  > lcov.info
cargo llvm-cov report --ignore-filename-regex=".+/macros/src/test_helpers.rs" --html
cargo llvm-cov report --ignore-filename-regex=".+/macros/src/test_helpers.rs" --summary-only
