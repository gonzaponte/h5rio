//! # h5rio
//!
//! `h5rio` is a small Rust library for writing and reading HDF5 datasets.
//! It focuses on three workflows:
//!
//! - append typed table records with [`TableHdf5Writer`]
//! - append fixed-shape `ndarray` values with [`ArrayHdf5Writer`]
//! - write one fixed-size chunked `ndarray` dataset with [`write_chunked_array`]
//!
//! It also provides complete read helpers, row-by-row table iteration,
//! entry-by-entry array iteration, and the [`h5type`] attribute macro for
//! HDF5-compatible table records.
//!
//! # Tutorials
//!
//! The repository includes runnable examples:
//!
//! - `examples/table_round_trip.rs`: write and read an appendable table.
//! - `examples/array_round_trip.rs`: write and read appendable arrays.
//! - `examples/client`: use tables and arrays from a downstream crate.
//!
//! # How-to Guides
//!
//! ## Write a table
//!
//! Use [`h5type`] for plain table row structs, then append rows with
//! [`TableHdf5Writer`].
//!
//! ```no_run
//! use std::rc::Rc;
//!
//! use h5rio::{h5type, read_table, TableHdf5Writer};
//! use hdf5_metno as hdf5;
//!
//! #[h5type]
//! struct Row {
//!     id: u64,
//!     value: f32,
//! }
//!
//! # fn main() -> hdf5::Result<()> {
//! let filename = "rows.h5";
//! let file = Rc::new(hdf5::File::create(filename)?);
//! let writer = TableHdf5Writer::<Row>::new(file, "/rows", 1024)?;
//!
//! writer.write(Row { id: 0, value: 1.5 })?;
//! writer.flush()?;
//!
//! let rows = read_table::<Row>(filename, "/rows")?;
//! assert_eq!(rows.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! ## Write appendable arrays
//!
//! Use [`ArrayHdf5Writer`] when each appended entry has the same fixed shape.
//! A writer created with `shape = vec![2, 3]` stores entries in a dataset with
//! shape `(n_entries, 2, 3)`.
//!
//! ## Write one fixed-size chunked array
//!
//! Use [`write_chunked_array`] when the complete array is already available and
//! the dataset should not have an extensible axis.
//!
//! ## Iterate over data
//!
//! Use [`iter_table`] to stream table rows and [`iter_array`] to stream array
//! entries along the leading axis.
//!
//! # Explanation
//!
//! Appendable writers use a resizable leading dataset axis and buffer entries
//! in memory until `chunk_size` entries are ready to be written. Calling
//! `flush()` writes any buffered entries immediately. Dropping a writer also
//! attempts to flush pending entries, but explicit `flush()` is preferred.
//!
//! [`write_chunked_array`] writes a fixed-size dataset with no extensible axes.
//! All writers use Blosc/Zlib compression through `hdf5-metno`.
//!
//! ## Design choices
//!
//! This crate is intentionally opinionated. It optimizes for a small set of
//! data-acquisition and simulation output patterns that are common and easy to
//! reason about:
//!
//! - datasets grow only through an appendable leading axis
//! - appended arrays have one fixed per-entry shape
//! - read helpers are simple, with full-dataset readers and row/entry iterators
//! - compression is fixed to Blosc/Zlib through `hdf5-metno`
//! - row types are ordinary Rust structs that implement `hdf5::H5Type`
//!
//! These choices keep the API compact and predictable, but they will not fit
//! every HDF5 use case. Applications that need arbitrary hyperslab updates,
//! multiple extensible axes, custom compression policies, or schema discovery
//! may need to use `hdf5-metno` directly.
//!
//! # API Reference
//!
//! The main entry points are [`TableHdf5Writer`], [`ArrayHdf5Writer`],
//! [`write_chunked_array`], [`read_table`], [`iter_table`], [`read_array`],
//! [`iter_array`], and [`h5type`].

mod reader;
pub use reader::read_table;
pub use reader::iter_table;
pub use reader::Hdf5TableIter;
pub use reader::read_array;
pub use reader::iter_array;
pub use reader::Hdf5ArrayIter;

mod array_writer;
mod table_writer;

pub use array_writer::write_chunked_array;
pub use array_writer::ArrayHdf5Writer;
pub use table_writer::TableHdf5Writer;


pub use h5rio_macros::h5type;

#[cfg(test)]
pub(crate) mod utils;
