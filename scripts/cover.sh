#!/bin/bash

cargo +nightly llvm-cov clean --workspace
cargo +nightly llvm-cov nextest --all-features --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --no-report
cargo +nightly llvm-cov         --all-features --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo +nightly llvm-cov nextest --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --no-report
cargo +nightly llvm-cov         --workspace --ignore-filename-regex=".+/macros/src/test_helpers.rs" --doc --no-report

cargo llvm-cov report --ignore-filename-regex=".+/macros/src/test_helpers.rs" --lcov  > lcov.info
cargo llvm-cov report --ignore-filename-regex=".+/macros/src/test_helpers.rs" --summary-only
