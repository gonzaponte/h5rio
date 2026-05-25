# -*-Makefile-*-

build:
    cargo build

test:
     cargo nextest run

verbose regexp:
     cargo nextest run --no-capture -E "test({{regexp}})"

clean:
    cargo clean
