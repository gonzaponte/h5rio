# -*-Makefile-*-

build:
    cargo build

test:
     cargo nextest run

test-macros:
     cargo nextest run -p h5rio_macros

verbose regexp:
     cargo nextest run --no-capture -E "test({{regexp}})"

clean:
    cargo clean

examples:
    cargo run --example array_round_trip
    cargo run --example table_round_trip
