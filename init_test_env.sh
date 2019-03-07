#!/bin/sh
export MU_ZEBU=$PWD
export ZEBU_BUILD=release
export CARGO_HOME=~/.cargo
export CC=clang
export CXX=clang++
export RUST_TEST_THREADS=1
export LD_LIBRARY_PATH=$MU_ZEBU/target/$ZEBU_BUILD/deps/:$LD_LIBRARY_PATH
