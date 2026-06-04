
mod reader;
pub use reader::read_table;
pub use reader::read_array;

mod array_writer;
mod table_writer;

pub use array_writer::write_chunked_array;
pub use array_writer::ArrayHdf5Writer;
pub use table_writer::TableHdf5Writer;


pub use h5rio_macros::h5type;

#[cfg(test)]
pub(crate) mod utils;
