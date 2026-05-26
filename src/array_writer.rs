use std::cell::RefCell;
use std::io::Result;
use std::rc::Rc;

use hdf5_metno::{self as hdf5, Error, Extent};
use hdf5_metno::filters::BloscShuffle;

use ndarray::{Array, ArrayViewD, Dimension, IxDyn};


pub struct ArrayHdf5Writer<T: hdf5::H5Type> {
    #[allow(dead_code)]
    _file     : Rc<hdf5::File>, // file needs to live while dataset lives, this is a way of ensuring that
    dataset   : hdf5::Dataset,
    chunk_size: usize, // chunk_size here refers to the flatten value
    cache     : RefCell<Vec<T>>, // cache is flattened
    shape     : Vec<usize>,
}

impl<T: hdf5::H5Type> ArrayHdf5Writer<T> {
    pub fn new( file       : Rc<hdf5::File>
              , dataset    : &str
              , chunk_size : usize
              , shape      : Vec<usize>
              ) -> hdf5::Result<Self> {
        if chunk_size == 0 {
            return Err(Error::Internal("Hdf5Writer::new: invalid chunk size 0".to_owned()));
        }

        let chunk_total = chunk_size * shape.iter().product::<usize>();

        let mut chunk_shape = vec![chunk_size];
        chunk_shape.extend_from_slice(&shape);

        let mut ds_shape = vec![Extent::resizable(0)];
        for s in shape.iter() {
            ds_shape.push(Extent::fixed(*s))
        }

        let dataset = file.new_dataset::<T>()
                          .chunk(chunk_shape.as_slice())
                          .shape(   ds_shape.as_slice())
                          .blosc_zlib(4, BloscShuffle::Byte)
                          .create(dataset)?;

        let cache = RefCell::new(Vec::with_capacity(chunk_total));
        let chunk_size = chunk_total;
        Ok(ArrayHdf5Writer{_file: file, dataset, chunk_size, cache, shape})
    }

    fn dump_cache(&self) -> Result<()> {
        let n_write = self.cache
                          .borrow()
                          .len()
                          .div_euclid(self.shape.iter().product()); // Why div_euclid???
        if n_write == 0 { return Ok(()) }

        let     size_before = self.dataset.shape()[0];
        let mut size_new    = self.shape.clone();
        size_new.insert(0, size_before + n_write);

        self.dataset.resize(size_new.as_slice())?;

        let mut data = vec![hdf5::SliceOrIndex::SliceCount {
            start: size_before,
            count: n_write,
            step : 1,
            block: 1,
        }];
        for _ in &self.shape {
            data.push(hdf5::SliceOrIndex::Unlimited {
                start: 0,
                step : 1,
                block: 1,
            });
        }
        let data = hdf5::Selection::from(hdf5::Hyperslab::from(data));

        let mut shape = self.shape.clone(); shape.insert(0, n_write);
        let shape = IxDyn(&shape);

        let cache = self.cache.borrow();
        let view = ArrayViewD::from_shape(shape, &cache[..])
            .expect("Cannot create array view with given shape");
        self.dataset.write_slice(view, data)?;

        drop(cache); // explicit drop to avoid holding immutable borrow
        self.cache.borrow_mut().clear();

        Ok(())
    }

    pub fn write<D: Dimension>(&self, item: Array<T,D>) -> Result<()> {
        self.cache.borrow_mut().extend(item.into_iter());

        if self.cache.borrow().len() == self.chunk_size {
            self.dump_cache()
        }
        else {
            Ok(())
        }
    }

    pub fn flush(&self) -> Result<()> {
        self.dump_cache()
    }

}

impl<T: hdf5::H5Type> Drop for ArrayHdf5Writer<T> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use hdf5_metno as hdf5;
    use ndarray::arr2;
    use pretty_assertions::assert_eq;

    use crate::read_array;
    use crate::utils::tempfile;

    #[test]
    fn new_valid() {
        let (_dir, filename) = tempfile("new_valid");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = ArrayHdf5Writer::<u16>::new(Rc::new(file), "/here", 123, vec![1,2,3]);
        assert!(writer.is_ok());
    }

    #[test]
    fn new_invalid_dataset_name() {
        let (_dir, filename) = tempfile("new_invalid_dataset_name");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = ArrayHdf5Writer::<u16>::new(Rc::new(file), "/", 123, vec![1,2,3]);
        assert!(writer.is_err());
    }

    #[test]
    fn new_invalid_chunksize() {
        let (_dir, filename) = tempfile("new_invalid_chunksize");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = ArrayHdf5Writer::<u16>::new(Rc::new(file), "/here", 0, vec![1,2,3]);
        assert!(writer.is_err());
    }

    #[test]
    fn round_trip_single() {
        let (_dir, filename) = tempfile("round_trip_single");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i32>::new(Rc::new(file), "/here", 1, vec![2,3]).unwrap();

        let data = arr2(&[[-1, 2, -3], [4, -5, 6]]);
        writer.write(data.clone()).unwrap();

        let read = read_array::<i32>(&filename, "/here").unwrap();

        assert_eq!(read.shape(), &[1, 2, 3]);

        assert_eq!(read[[0,0,0]], data[[0,0]]);
        assert_eq!(read[[0,0,1]], data[[0,1]]);
        assert_eq!(read[[0,0,2]], data[[0,2]]);
        assert_eq!(read[[0,1,0]], data[[1,0]]);
        assert_eq!(read[[0,1,1]], data[[1,1]]);
        assert_eq!(read[[0,1,2]], data[[1,2]]);
    }

    #[test]
    fn round_trip_double() {
        let (_dir, filename) = tempfile("round_trip_double");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i32>::new(Rc::new(file), "/here", 1, vec![2,3]).unwrap();

        let data0 = arr2(&[[- 1,  2, - 3], [ 4, - 5,  6]]);
        let data1 = arr2(&[[-11, 22, -33], [44, -55, 66]]);
        writer.write(data0.clone()).unwrap();
        writer.write(data1.clone()).unwrap();

        let read = read_array::<i32>(&filename, "/here").unwrap();

        assert_eq!(read.shape(), &[2, 2, 3]);

        assert_eq!(read[[0,0,0]], data0[[0,0]]);
        assert_eq!(read[[0,0,1]], data0[[0,1]]);
        assert_eq!(read[[0,0,2]], data0[[0,2]]);
        assert_eq!(read[[0,1,0]], data0[[1,0]]);
        assert_eq!(read[[0,1,1]], data0[[1,1]]);
        assert_eq!(read[[0,1,2]], data0[[1,2]]);
        assert_eq!(read[[1,0,0]], data1[[0,0]]);
        assert_eq!(read[[1,0,1]], data1[[0,1]]);
        assert_eq!(read[[1,0,2]], data1[[0,2]]);
        assert_eq!(read[[1,1,0]], data1[[1,0]]);
        assert_eq!(read[[1,1,1]], data1[[1,1]]);
        assert_eq!(read[[1,1,2]], data1[[1,2]]);
    }

    #[test]
    fn flush_on_drop() {
        // Write less entries than chunk size and verify that it is still written
        let (_dir, filename) = tempfile("flush_on_drop");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i64>::new(Rc::new(file), "/here", 5, vec![2,3]).unwrap();

        let data = arr2(&[[-1, 2, -3], [4, -5, 6]]);
        writer.write(data.clone()).unwrap(); // not actually written because of cache

        let read = read_array::<i64>(&filename, "/here").unwrap();
        assert_eq!(read.shape(), &[0, 2, 3]);

        drop(writer);
        let read = read_array::<i64>(&filename, "/here").unwrap();
        assert_eq!(read.shape(), &[1, 2, 3]);

    }

}
