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
//! <br>
//!
//! # Tutorials
//!
//! The repository includes runnable examples:
//!
//! - `examples/table_round_trip.rs`: write and read an appendable table.
//! - `examples/array_round_trip.rs`: write and read appendable arrays.
//! - `examples/client`: use tables and arrays from a downstream crate.
//!
//! <br>
//!
//! # How-to Guides
//!
//! ## Choose an API
//!
//! Use [`TableHdf5Writer`] when each entry is one typed record, such as a hit,
//! event, log row, or simulation step summary. The dataset grows as a
//! one-dimensional table with shape `(n_rows,)`.
//!
//! Use [`ArrayHdf5Writer`] when each entry is an `ndarray` with the same shape,
//! such as a waveform, image, response map, or per-event matrix. The dataset
//! grows along a leading axis with shape `(n_entries, *item_shape)`.
//!
//! Use [`write_chunked_array`] when you already have the whole array and want
//! to write exactly that fixed-size dataset. It does not create an appendable
//! axis.
//!
//! Use [`read_table`] or [`read_array`] when the full dataset fits comfortably
//! in memory. Use [`iter_table`] or [`iter_array`] when you want to keep memory
//! bounded and process one table row or one leading-axis array entry at a time.
//!
//! ## Handle validation errors
//!
//! Most APIs return `hdf5::Result<_>`. Errors can come from HDF5 itself, from
//! file or dataset paths, or from `h5rio` validation before data are buffered
//! or written.
//!
//! The main validation rules are:
//!
//! - `chunk_size` must be greater than zero for [`TableHdf5Writer`] and
//!   [`ArrayHdf5Writer`].
//! - [`ArrayHdf5Writer::new`] requires every configured item-shape dimension
//!   to be nonzero.
//! - [`ArrayHdf5Writer::write`] requires every appended array to have exactly
//!   the configured item shape.
//! - [`write_chunked_array`] requires `chunk_shape` to have the same rank as
//!   the array being written, and every chunk dimension must be nonzero.
//! - [`iter_table`] requires a one-dimensional dataset with shape `(n_rows,)`.
//! - [`iter_array`] requires a non-scalar dataset and iterates over its leading
//!   axis.
//!
//! Buffered writers also attempt to flush when dropped, but explicit
//! [`TableHdf5Writer::flush`] or [`ArrayHdf5Writer::flush`] is preferred
//! because it returns any write error directly.
//!
//! ## Define table row types
//!
//! Use [`h5type`] for plain table-row structs whose fields are themselves
//! HDF5-compatible.
//!
//! ```no_run
//! use h5rio::h5type;
//!
//! #[h5type]
//! struct Hit {
//!     event_id: u64,
//!     sensor_id: u32,
//!     charge: f32,
//! }
//! ```
//!
//! The macro expands the struct definition with:
//!
//! ```ignore
//! #[derive(::hdf5_metno::H5Type, Clone, PartialEq, Debug)]
//! #[repr(C)]
//! ```
//!
//! It uses the explicit `hdf5_metno` crate path for the derive. You do not
//! need to alias `hdf5_metno` as `hdf5` for the macro itself.
//!
//! The macro is deliberately small: it does not inspect or rewrite fields, and
//! it does not provide schema migration. Changing field names, field order, or
//! field types changes the HDF5 compound type expected by readers and writers.
//! Avoid duplicating the generated derives on the same struct, because that
//! would create duplicate trait implementations.
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
//! writer.write(Row { id: 1, value: 6.7 })?;
//! writer.flush()?;
//!
//! let rows = read_table::<Row>(filename, "/rows")?;
//! assert_eq!(rows.len(), 2);
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
//! ```no_run
//! use std::rc::Rc;
//!
//! use h5rio::{read_array, ArrayHdf5Writer};
//! use hdf5_metno as hdf5;
//! use ndarray::array;
//!
//! # fn main() -> hdf5::Result<()> {
//! let filename = "waveforms.h5";
//! let file = Rc::new(hdf5::File::create(filename)?);
//!
//! // Buffer two entries per chunk. Each entry must have shape (2, 3).
//! let writer = ArrayHdf5Writer::<f32>::new(file, "/waves", 2, vec![2, 3])?;
//!
//! writer.write(array![
//!     [0.1, 1.2, 2.3],
//!     [3.4, 4.5, 5.6],
//! ])?;
//!
//! writer.write(array![
//!     [-0.1, -1.2, -2.3],
//!     [-3.4, -4.5, -5.6],
//! ])?;
//!
//! writer.flush()?;
//!
//! let data = read_array::<f32>(filename, "/waves")?;
//! assert_eq!(data.shape(), &[2, 2, 3]);
//! # Ok(())
//! # }
//! ```
//!
//! The `chunk_size` argument counts appended entries, not scalar elements. A
//! shape mismatch in [`ArrayHdf5Writer::write`] is rejected before the array is
//! buffered.
//!
//! ## Write one fixed-size chunked array
//!
//! Use [`write_chunked_array`] when the complete array is already available and
//! the dataset should not have an extensible axis.
//!
//! ```no_run
//! use std::rc::Rc;
//!
//! use h5rio::{read_array, write_chunked_array};
//! use hdf5_metno as hdf5;
//! use ndarray::array;
//!
//! # fn main() -> hdf5::Result<()> {
//! let filename = "image.h5";
//! let file = Rc::new(hdf5::File::create(filename)?);
//! let image = array![
//!     [0.0f32, 0.5, 1.0],
//!     [1.0   , 0.5, 0.0],
//! ];
//!
//! // The dataset shape is exactly (2, 3); the chunk shape has the same rank.
//! write_chunked_array(file, "/image", vec![1, 3], &image)?;
//!
//! let data = read_array::<f32>(filename, "/image")?;
//! assert_eq!(data.shape(), &[2, 3]);
//! # Ok(())
//! # }
//! ```
//!
//! `chunk_shape` must have the same rank as the array being written and must
//! not contain zero dimensions.
//!
//! ## Iterate over data
//!
//! Use [`iter_table`] to stream table rows and [`iter_array`] to stream array
//! entries along the leading axis.
//!
//! ```no_run
//! use h5rio::{h5type, iter_array, iter_table};
//!
//! #[h5type]
//! struct Row {
//!     id: u64,
//!     value: f32,
//! }
//!
//! # fn main() -> hdf5_metno::Result<()> {
//! for row in iter_table::<Row>("rows.h5", "/rows")? {
//!     let row = row?;
//!     println!("row {}: {}", row.id, row.value);
//! }
//!
//! for entry in iter_array::<f32>("waveforms.h5", "/waves")? {
//!     let entry = entry?;
//!     println!("array entry shape: {:?}", entry.shape());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [`iter_table`] requires a one-dimensional table dataset. [`iter_array`]
//! requires a non-scalar dataset and yields one array per leading-axis index;
//! for a dataset with shape `(n_entries, 2, 3)`, each yielded entry has shape
//! `(2, 3)`.
//!
//! <br>
//!
//! # Explanation
//!
//! Appendable writers use a resizable leading dataset axis and buffer entries
//! in memory until `chunk_size` entries are ready to be written. Calling
//! `flush()` writes any buffered entries immediately. Dropping a writer also
//! attempts to flush pending entries, but explicit `flush()` is preferred.
//!
//! [`read_table`] and [`read_array`] are eager readers: they load complete
//! datasets into memory before returning. This is the simplest option for
//! small and medium datasets, but peak memory use scales with the full dataset
//! size.
//!
//! [`iter_table`] and [`iter_array`] are lazy readers: they keep the file open
//! and read one item per iteration. For [`iter_table`], one item is one row.
//! For [`iter_array`], one item is one slice along the dataset's leading axis,
//! so memory use scales with one entry rather than the full dataset.
//!
//! [`write_chunked_array`] writes a fixed-size dataset with no extensible axes.
//! All writers use Blosc/Zlib compression through `hdf5-metno`.
//!
//! ## Why `#[h5type]` exists
//!
//! HDF5 table rows are stored as compound values. Rust's default struct layout
//! is not a stable external data layout, so table row types need a C-compatible
//! representation and an [`hdf5_metno::H5Type`] implementation. The [`h5type`]
//! macro adds those pieces in one place, keeping application code focused on
//! the record fields while making the storage contract somewhat explicit.
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
//! <br>
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
