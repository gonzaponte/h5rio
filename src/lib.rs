
//! A small Rust library for writing and reading HDF5 datasets.
//!
//! `h5rio` focuses on two common output patterns:
//!
//! - append typed table records with [`TableHdf5Writer`]
//! - append fixed-shape `ndarray` values with [`ArrayHdf5Writer`]
//! - write one fixed-size chunked `ndarray` dataset with
//!   [`write_chunked_array`]
//!
//! It also provides complete read helpers, entry-by-entry table and array
//! iteration, and the [`h5type`] attribute macro for HDF5-compatible table
//! records.
//!
//! # Example
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
//! Appendable writers use a resizable leading dataset axis.
//! [`write_chunked_array`] writes a fixed-size dataset with no extensible axes.
//! All writers use Blosc/Zlib compression through `hdf5-metno`.

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
