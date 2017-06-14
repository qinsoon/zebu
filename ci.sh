set -e
if [ -z "$MU_ZEBU" ]
then
	export MU_ZEBU=$PWD
fi
export MU_LOG_LEVEL=none
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=0
export PYTHONPATH="$MU_ZEBU/tests/test_jit/mu-client-pypy/:$MU_ZEBU/tests/test_jit/RPySOM/src"
export LD_LIBRARY_PATH="$MU_ZEBU/tests/test_jit/:$MU_ZEBU/tests/test_jit"
export ZEBU_BUILD=release

rm -rf $MU_ZEBU/emit
rm -rf $MU_ZEBU/tests/test_jit/emit

#cargo clean
cargo test --release --no-run --color=always 2>&1 | tee build_out.txt

/usr/bin/time -f "finished in %e secs" -a -o cargo_test_out.txt ./test-release --color=always 2>/dev/null | tee cargo_test_out.txt

cd $MU_ZEBU/tests/test_jit/mu-client-pypy
git pull

cd $MU_ZEBU/tests/test_jit/RPySOM
git pull

cd $MU_ZEBU/tests/test_jit/
ZEBU_BUILD=release LD_LIBRARY_PATH=. PYTHONPATH=mu-client-pypy:RPySOM/src pytest test_*.py -v --color=yes 2>&1 | tee $MU_ZEBU/pytest_out.txt

