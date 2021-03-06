#!/usr/bin/env bash
export MU_ZEBU=$PWD/../../../mu-impl-fast/
export ZEBU_BUILD=release
export DYLD_LIBRARY_PATH=$PWD
export LD_LIBRARY_PATH=$MU_ZEBU/target/$ZEBU_BUILD/deps:$LD_LIBRARY_PATH
export LD_LIBRARY_PATH=./emit:$LD_LIBRARY_PATH
export usess=none
#export PYTHONPATH=mu-client-pypy:RPySOM/src
export RPYSOM=RPySOM
export CC=clang
export SPAWN_PROC=1
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=1
