# h5rio

[![Test suite](https://github.com/gonzaponte/h5rio/actions/workflows/test.yml/badge.svg)](https://github.com/gonzaponte/h5rio/actions/workflows/test.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)

A small Rust library for writing and reading HDF5 datasets.

`h5rio` provides a compact interface for three common data-acquisition and
simulation output patterns:

- **Tables**: append typed records to a one-dimensional HDF5 dataset.
- **Arrays**: append fixed-shape `ndarray` values along a resizable first axis.
- **Fixed-size arrays**: write one fixed-size `ndarray` dataset with chunking and
  compression, without an extensible axis.

Datasets are buffered in memory and written in chunks, using Blosc/Zlib
compression through [`hdf5-metno`](https://github.com/matthias314/hdf5-metno).
A convenience attribute macro, `#[h5type]`, is provided for defining
HDF5-compatible table records.

## Choosing an API

Use `TableHdf5Writer<T>` when each entry is one typed record, such as a hit,
event, log row, or simulation step summary.

Use `ArrayHdf5Writer<T>` when each entry is an `ndarray` with the same shape,
such as a waveform, image, response map, or per-event matrix.

Use `write_chunked_array` when you already have the whole array and want to
write exactly that fixed-size dataset.

Use `read_table` or `read_array` when the full dataset fits comfortably in
memory. Use `iter_table` or `iter_array` when you want to keep memory bounded
and process one table row or one leading-axis array entry at a time.

## Installation

```toml
[dependencies]
h5rio = "0.2.0"

# Needed to create/open HDF5 files and by the #[h5type] macro expansion.
hdf5_metno = { package = "hdf5-metno", version = "0.12.3", features = ["blosc-zlib"] }

# Needed when writing ndarray values.
ndarray = "0.17.2"
```

The library requires an HDF5 installation available to `hdf5-metno`. The
repository includes a Nix development shell that provides HDF5 and the pinned
Rust toolchain.

## Documentation

Full API documentation and how-to guides are available on
[`docs.rs`](https://docs.rs/h5rio/latest/h5rio/).

Runnable examples are included in the repository:

- [`examples/table_round_trip.rs`](examples/table_round_trip.rs)
- [`examples/array_round_trip.rs`](examples/array_round_trip.rs)
- [`examples/client`](examples/client)

## Quick start

```rust
use std::rc::Rc;

use h5rio::{h5type, read_table, TableHdf5Writer};
use hdf5_metno as hdf5;


#[h5type]
struct Hit {
    event_id: u64,
    sensor_id: u32,
    charge: f32,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "hits.h5";
    let file = Rc::new(hdf5::File::create(filename)?);

    // Flush every 1024 rows. The dataset is created at /hits.
    let writer = TableHdf5Writer::<Hit>::new(file, "/hits", 1024)?;

    writer.write(Hit {
        event_id: 0,
        sensor_id: 12,
        charge: 18.4,
    })?;

    writer.write(Hit {
        event_id: 0,
        sensor_id: 19,
        charge: 4.7,
    })?;

    writer.flush()?;

    let hits = read_table::<Hit>(filename, "/hits")?;
    println!("Read {} hits", hits.len());

    Ok(())
}
```

## Development

The recommended development environment is provided by the flake. It includes
the pinned Rust toolchain, HDF5, `cargo-nextest`, `just`, `bacon`, and
Rust Analyzer support.

```bash
nix develop
just build
just test
```

Without Nix, install HDF5 and a compatible Rust toolchain, then run:

```bash
cargo build
cargo test
```

The repository pins Rust `1.95.0` in `rust-toolchain.toml`. The project's
`justfile` uses `cargo nextest` for its test recipes.

For contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the terms of the [GNU General Public License
v3.0](LICENSE).
