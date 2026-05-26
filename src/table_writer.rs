use std::io::Result;
use std::rc::Rc;

use hdf5_metno as hdf5;

use ndarray::arr0;

use crate::ArrayHdf5Writer;

pub struct TableHdf5Writer<T: hdf5::H5Type>(ArrayHdf5Writer<T>);

impl<T: hdf5::H5Type> TableHdf5Writer<T> {
    pub fn new(file: Rc<hdf5::File>, dataset: &str, chunk_size: usize) -> hdf5::Result<Self> {
        ArrayHdf5Writer::new(file, dataset, chunk_size, vec![]).map(Self)
    }

    pub fn flush(&self) -> Result<()> {
        self.0.flush()
    }

    pub fn write(&self, value: T) -> Result<()> {
        self.0.write(arr0(value))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hdf5_metno as hdf5;
    use pretty_assertions::assert_eq;
    use crate::read_table;
    use crate::utils::tempfile;

    // This is meant to be done with #[h5type], but here we keep things as
    // independent as possible
    #[derive(hdf5::H5Type, Clone, PartialEq, Debug)]
    #[repr(C)]
    pub struct Dummy {
        a: i64,
        b: f32,
    }

    type DummyWriter = TableHdf5Writer::<Dummy>;

    #[test]
    fn new_valid() {
        let (_dir, filename) = tempfile("new_valid");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/here", 123);
        assert!(writer.is_ok());
    }

    #[test]
    fn new_invalid_dataset_name() {
        let (_dir, filename) = tempfile("new_invalid_dataset_name");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/", 123);
        assert!(writer.is_err());
    }

    #[test]
    fn new_invalid_chunksize() {
        let (_dir, filename) = tempfile("new_invalid_chunksize");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/here", 0);
        assert!(writer.is_err());
    }

    #[test]
    fn round_trip_single() {
        let (_dir, filename) = tempfile("round_trip_single");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/here", 1).unwrap();

        let data = Dummy{a: 42, b: 3.14};
        writer.write(data.clone()).unwrap();

        let read = read_table::<Dummy>(&filename, "/here").unwrap();

        assert_eq!(read.len(), 1);
        assert_eq!(read[0], data);
    }

    #[test]
    fn round_trip_double() {
        let (_dir, filename) = tempfile("round_trip_double");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/here", 1).unwrap();

        let data0 = Dummy{a: 42, b: 3.14};
        let data1 = Dummy{a: 99, b: 2.72};
        writer.write(data0.clone()).unwrap();
        writer.write(data1.clone()).unwrap();

        let read = read_table::<Dummy>(&filename, "/here").unwrap();

        assert_eq!(read.len(), 2);
        assert_eq!(read[0], data0);
        assert_eq!(read[1], data1);
    }

    #[test]
    fn flush_on_drop() {
        // Write less entries than chunk size and verify that it is still written
        let (_dir, filename) = tempfile("flush_on_drop");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = DummyWriter::new(Rc::new(file), "/here", 5).unwrap();

        let data = Dummy{a: 42, b: 3.14};
        writer.write(data.clone()).unwrap(); // not actually written because of cache

        let read = read_table::<Dummy>(&filename, "/here").unwrap();
        assert_eq!(read.len(), 0);

        drop(writer);
        let read = read_table::<Dummy>(&filename, "/here").unwrap();
        assert_eq!(read.len(), 1);
    }

}
