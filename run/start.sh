ulimit -n 10000
export RUST_BACKTRACE=1
./mazze --config hydra.toml 2> stderr.txt
