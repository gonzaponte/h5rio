# h5rio

[![Test suite](https://github.com/gonzaponte/h5rio/actions/workflows/test.yml/badge.svg)](https://github.com/gonzaponte/h5rio/actions/workflows/test.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)

A small Rust library for writing and reading HDF5 datasets.

`h5rio` provides a compact interface for two common data-acquisition and
simulation output patterns:

- **Tables**: append typed records to a one-dimensional HDF5 dataset.
- **Arrays**: append fixed-shape `ndarray` values along a resizable first axis.
- **Fixed-size arrays**: write one fixed-size `ndarray` dataset with chunking and
  compression, without an extensible axis.

Datasets are buffered in memory and written in chunks, using Blosc/Zlib
compression through [`hdf5-metno`](https://github.com/matthias314/hdf5-metno).
A convenience attribute macro, `#[h5type]`, is provided for defining
HDF5-compatible table records.

## Features

- Append-only HDF5 table writer for compound Rust types.
- Append-only HDF5 array writer for fixed-shape `ndarray` entries.
- One-shot chunked array writer for fixed-size `ndarray` datasets.
- Configurable buffering through the number of entries stored per chunk.
- Shape validation for array writers, appended arrays, and chunked writes.
- Read helpers for complete table and array datasets.
- Iterator helper for reading array datasets entry by entry.
- `#[h5type]` macro for HDF5-compatible row types.

## Data model

`TableHdf5Writer<T>` creates a dataset with shape

```text
(n_rows,)
```

where each call to `write(value)` appends one record of type `T`.

`ArrayHdf5Writer<T>` creates a dataset with shape

```text
(n_entries, *item_shape)
```

where `item_shape` is fixed at construction time and the leading dimension
grows as arrays are appended. For example, a writer created with
`shape = vec![2, 3]` stores successive `2 x 3` arrays in a dataset with shape
`(n_entries, 2, 3)`.

Every configured `item_shape` dimension must be nonzero. Every appended array
must have exactly that configured shape; mismatched array shapes are rejected
before the writer buffers the value.

Both appendable writers use a resizable leading axis and Blosc/Zlib
compression. `write_chunked_array` writes a fixed-size dataset with no
extensible axes, using the chunk shape provided by the caller.

## Installation

To use the current release:

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

## Quick start: writing a table

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

    // Optional: data are also flushed when the writer is dropped.
    writer.flush()?;

    let hits = read_table::<Hit>(filename, "/hits")?;

    println!("Read {} hits", hits.len());

    Ok(())
}
```

The `#[h5type]` attribute expands the struct definition with the derives and
representation required for table storage:

```rust
#[derive(::hdf5_metno::H5Type, Clone, PartialEq, Debug)]
#[repr(C)]
```

The macro uses the explicit `hdf5_metno` crate path for the derive. You do not
need to alias `hdf5_metno` as `hdf5` for the macro itself, although the examples
do so when creating files through `hdf5::File`.

## Writing fixed-shape arrays

`ArrayHdf5Writer` is useful for storing successive waveforms, images,
response maps, or any stream of arrays with the same shape. Each `write` call
appends one array whose shape must match the shape passed to `new`.

```rust
use std::rc::Rc;

use h5rio::{read_array, ArrayHdf5Writer};
use hdf5_metno as hdf5;
use ndarray::arr2;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "waveforms.h5";
    let file = Rc::new(hdf5::File::create(filename)?);

    // Each appended entry has shape (2, 3); two entries are buffered per chunk.
    let writer = ArrayHdf5Writer::<f32>::new(
        file,
        "/waveforms",
        2,
        vec![2, 3],
    )?;

    writer.write(arr2(&[
        [0.0, 1.0, 0.5],
        [0.2, 0.7, 0.1],
    ]))?;

    writer.write(arr2(&[
        [1.1, 1.4, 1.0],
        [0.8, 0.3, 0.0],
    ]))?;

    writer.flush()?;

    let waveforms = read_array::<f32>(filename, "/waveforms")?;

    assert_eq!(waveforms.shape(), &[2, 2, 3]);

    Ok(())
}
```

## Iterating arrays

`iter_array` reads an array dataset one entry at a time along the leading
dimension. For a dataset with shape `(n_entries, 2, 3)`, each item yielded by
the iterator has shape `(2, 3)`. Scalar datasets cannot be iterated this way and
are rejected with an error.

```rust
use h5rio::iter_array;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    for entry in iter_array::<f32>("waveforms.h5", "/waveforms")? {
        let waveform = entry?;
        println!("Read entry with shape {:?}", waveform.shape());
    }

    Ok(())
}
```

## Writing fixed-size chunked arrays

Use `write_chunked_array` when you already have the complete array and do not
need an appendable leading axis. The dataset shape is exactly the shape of the
array being written; `chunk_shape` controls HDF5 chunking and must have the same
rank as the array with no zero dimensions.

```rust
use std::rc::Rc;

use h5rio::{read_array, write_chunked_array};
use hdf5_metno as hdf5;
use ndarray::array;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "image.h5";
    let file = Rc::new(hdf5::File::create(filename)?);
    let image = array![
        [0.0, 0.5, 1.0],
        [1.0, 0.5, 0.0],
    ];

    write_chunked_array(file, "/image", vec![1, 3], &image)?;

    let read = read_array::<f32>(filename, "/image")?;
    assert_eq!(read.shape(), &[2, 3]);

    Ok(())
}
```

## API overview

### Writers

| Type | Purpose | Main methods |
| --- | --- | --- |
| `TableHdf5Writer<T>` | Append scalar records of an HDF5-compatible type | `new`, `write`, `flush` |
| `ArrayHdf5Writer<T>` | Append fixed-shape `ndarray::Array` entries | `new`, `write`, `flush` |
| `write_chunked_array` | Write one fixed-size chunked `ndarray::Array` dataset | function |

The `chunk_size` constructor argument is the number of appended **entries**
buffered before a write to disk. For table datasets, one entry is one record.
For array datasets, one entry is one full array with the configured
`item_shape`. The configured array shape must not contain zero dimensions.

`write_chunked_array` also validates its `chunk_shape`: it must have the same
rank as the array being written and all chunk dimensions must be nonzero.

### Readers

```rust
pub fn read_table<T>(filename: &str, dataset: &str) -> hdf5::Result<Vec<T>>
pub fn read_array<T>(filename: &str, dataset: &str) -> hdf5::Result<ArrayD<T>>
pub fn iter_array<T>(filename: &str, dataset: &str) -> hdf5::Result<Hdf5ArrayIter<T>>
```

`read_table` and `read_array` load complete datasets into memory. `iter_array`
keeps the file open and yields one array entry at a time.

### Macro

```rust
#[h5type]
struct Row {
    value: f64,
}
```

`#[h5type]` is intended for plain record structs used with
`TableHdf5Writer<T>` and `read_table<T>`.

## Development

### With Nix

The recommended development environment is provided by the flake. It includes
the pinned Rust toolchain, HDF5, `cargo-nextest`, `just`, `bacon`, and
Rust Analyzer support.

```bash
nix develop
just build
just test
```

### Without Nix

Install HDF5 and a compatible Rust toolchain, then run:

```bash
cargo build
cargo test
```

The repository pins Rust `1.95.0` in `rust-toolchain.toml`. The project's
`justfile` uses `cargo nextest` for its test recipes.

For contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Testing and continuous integration

Tests cover:

- construction of valid and invalid writers;
- table and array round trips;
- rejection of arrays with shapes that do not match the writer configuration;
- rejection of invalid writer shapes and chunk shapes;
- flushing incomplete chunks when writers are dropped;
- complete and entry-by-entry array reading;
- rejection of scalar datasets passed to `iter_array`;
- the derives and C-compatible memory layout generated by `#[h5type]`.

On every push and pull request, the GitHub Actions workflow builds and tests
the project inside the Nix development environment with warnings treated as
errors.

## License

This project is licensed under the terms of the [GNU General Public License
v3.0](LICENSE).
