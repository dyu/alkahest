
build_example() {
    cargo build --release --example $1 --features='derive alloc'
}

build_example profile
build_example simple
build_example test

cd benchmark && cargo build --release --bench benchmark