#!/bin/sh

set -e

sudo RUST_TEST_THREADS=1 RUST_BACKTRACE=1 RUST_LOG=debug,rustc=warn,cargo=warn RTE_SDK=/home/flier/dpdk-16.04 cargo test
