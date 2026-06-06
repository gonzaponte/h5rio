use std::cell::RefCell;
use std::rc::Rc;

use hdf5_metno as hdf5;
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
            let msg = "Hdf5Writer::new: invalid chunk size 0".to_owned();
            return Err(hdf5::Error::Internal(msg));
        }

        if shape.iter().any(|s| *s == 0) {
            let msg = format!("ArrayHdf5Writer::new: invalid array shape {shape:?}");
            return Err(hdf5::Error::Internal(msg));
        }

        let chunk_total = chunk_size * shape.iter().product::<usize>();

        let mut chunk_shape = vec![chunk_size];
        chunk_shape.extend_from_slice(&shape);

        let mut ds_shape = vec![hdf5::Extent::resizable(0)];
        for s in shape.iter() {
            ds_shape.push(hdf5::Extent::fixed(*s))
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

    fn dump_cache(&self) -> hdf5::Result<()> {
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
            .map_err(|error| {
                let msg = format!("ArrayHdf5Writer: cannot create array view from cache: {error}");
                hdf5::Error::Internal(msg)
            })?;
        self.dataset.write_slice(view, data)?;

        drop(cache); // explicit drop to avoid holding immutable borrow
        self.cache.borrow_mut().clear();

        Ok(())
    }

    pub fn write<D: Dimension>(&self, item: Array<T,D>) -> hdf5::Result<()> {
        if item.shape() != self.shape.as_slice() {
            return Err(hdf5::Error::Internal(
                format!(
                    "ArrayHdf5Writer::write: invalid array shape {:?}, expected {:?}",
                    item.shape(),
                    self.shape
                ),
            ));
        }

        self.cache.borrow_mut().extend(item.into_iter());

        if self.cache.borrow().len() == self.chunk_size {
            self.dump_cache()
        }
        else {
            Ok(())
        }
    }

    pub fn flush(&self) -> hdf5::Result<()> {
        self.dump_cache()
    }

}

impl<T: hdf5::H5Type> Drop for ArrayHdf5Writer<T> {
    fn drop(&mut self) {
        if self.cache.borrow().is_empty() { return; }

        let item_size = self.shape.iter().product::<usize>();
        let n_entries = self.cache.borrow().len() / item_size;

        eprintln!("ArrayHdf5Writer: dropping writer with {n_entries} buffered\
                   entries; flushing now. To avoid this warning flush the\
                   writer explicitly.");

        if let Err(error) = self.flush() {
            eprintln!("ArrayHdf5Writer: failed to flush buffered entries on\
                       drop: {error}");
        }
    }
}


pub fn write_chunked_array<T, D>( file        : Rc<hdf5::File>
                                , dataset     : &str
                                , chunk_shape : Vec<usize>
                                , array       : &Array<T, D>) -> hdf5::Result<()>
where T: hdf5::H5Type,
      D: Dimension,
{
    if chunk_shape.len() != array.ndim() || chunk_shape.iter().any(|c| *c == 0) {
        let msg = format!("write_chunked_array(): invalid chunk shape {:?} for\
                           array shape {:?}", chunk_shape, array.shape());
        return Err(hdf5::Error::Internal(msg));
    }

    let ds_shape = array.shape()
                        .iter()
                        .map(|s| hdf5::Extent::fixed(*s))
                        .collect::<Vec<_>>();
    let dataset = file.new_dataset::<T>()
                      .chunk(chunk_shape.as_slice())
                      .shape(   ds_shape.as_slice())
                      .blosc_zlib(4, BloscShuffle::Byte)
                      .create(dataset)?;

    dataset.write(array.view())
}


#[cfg(test)]
mod tests {
    use super::*;

    use hdf5_metno as hdf5;
    use ndarray::array;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
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

    #[rstest]
    #[case(vec![0, 1, 1])]
    #[case(vec![1, 0, 1])]
    #[case(vec![1, 1, 0])]
    fn new_invalid_shape(#[case] shape: Vec<usize>) {
        let (_dir, filename) = tempfile("new_invalid_shape");
        let file             = hdf5::File::create(filename).unwrap();
        let writer           = ArrayHdf5Writer::<u16>::new(Rc::new(file), "/here", 123, shape);
        assert!(matches!(writer, Err(hdf5::Error::Internal(_))));
    }

    #[test]
    fn round_trip_single() {
        let (_dir, filename) = tempfile("round_trip_single");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i32>::new(Rc::new(file), "/here", 1, vec![2,3]).unwrap();

        let data = array![ [-1,  2, -3]
                         , [ 4, -5,  6] ];
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

        let data0 = array![ [- 1,  2, - 3], [ 4, - 5,  6] ];
        let data1 = array![ [-11, 22, -33], [44, -55, 66] ];
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
        // Write less entries than chunk size and verify that they are still written
        let (_dir, filename) = tempfile("flush_on_drop");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i64>::new(Rc::new(file), "/here", 5, vec![2,3]).unwrap();

        let data = array![ [-1, 2, -3], [4, -5, 6] ];
        writer.write(data.clone()).unwrap(); // not actually written because of cache

        let read = read_array::<i64>(&filename, "/here").unwrap();
        assert_eq!(read.shape(), &[0, 2, 3]);

        drop(writer);
        let read = read_array::<i64>(&filename, "/here").unwrap();
        assert_eq!(read.shape(), &[1, 2, 3]);

    }

    #[test]
    fn drop_does_not_panic_when_flush_fails() {
        let (_dir, filename) = tempfile("drop_does_not_panic_when_flush_fails");
        let file             = hdf5::File::create(filename).unwrap();
        let mut writer       = ArrayHdf5Writer::<i64>::new(Rc::new(file), "/here", 5, vec![2,3]).unwrap();

        writer.write(array![ [-1, 2, -3], [4, -5, 6] ]).unwrap();

        // intentionally corrupt the writer to induce a failure on write
        writer.shape = vec![3, 2];

        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(writer);
        }));

        assert!(out.is_ok());
    }

    #[test]
    fn write_invalid_shape() {
        let (_dir, filename) = tempfile("write_invalid_shape");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let writer           = ArrayHdf5Writer::<i32>::new(Rc::new(file), "/here", 1, vec![2,3]).unwrap();

        let invalid = array![ [1, 2], [3, 4] ];
        let out     = writer.write(invalid);

        assert!(matches!(out, Err(hdf5::Error::Internal(_))));

        let valid = array![ [10, 20, 30], [40, 50, 60] ];
        writer.write(valid.clone()).unwrap();

        let read = read_array::<i32>(&filename, "/here").unwrap();

        assert_eq!(read.shape(), &[1, 2, 3]);
        assert_eq!(read[[0,0,0]], valid[[0,0]]);
        assert_eq!(read[[0,0,1]], valid[[0,1]]);
        assert_eq!(read[[0,0,2]], valid[[0,2]]);
        assert_eq!(read[[0,1,0]], valid[[1,0]]);
        assert_eq!(read[[0,1,1]], valid[[1,1]]);
        assert_eq!(read[[0,1,2]], valid[[1,2]]);
    }

    #[rstest]
    #[case(vec![1])]       // invalid shape
    #[case(vec![1, 1])]    //
    #[case(vec![0, 1, 1])] // can't have zeros
    #[case(vec![1, 0, 1])] //
    #[case(vec![1, 1, 0])] //
    fn write_carray_invalid_chunk_shape(#[case] chunk_shape: Vec<usize>) {
        let array = array![ [ [0, 1] ,
                              [2, 3] ],

                            [ [4, 5] ,
                              [6, 7] ],
                          ]; // shape (2, 2, 2)
        let (_dir, filename) = tempfile("flush_on_drop");
        let file             = hdf5::File::create(filename.clone()).unwrap();
        let out              = write_chunked_array(Rc::new(file), "/here", chunk_shape, &array);
        assert!(matches!(out, Err(hdf5::Error::Internal(_))));
        assert!(out.unwrap_err().to_string().contains("invalid chunk shape"));
    }

    #[test]
    fn write_carray_invalid_dataset_name() {
        let array            = array![ 1, 2, 3 ];
        let (_dir, filename) = tempfile("write_carray_invalid_dataset_name");
        let file             = hdf5::File::create(filename).unwrap();
        let out              = write_chunked_array(Rc::new(file), "", vec![1], &array);
        assert!(out.is_err());
    }

    #[rstest]
    #[case(vec![1, 2, 3         ], vec![3])]
    #[case(vec![1, 2, 3, 4, 5, 6], vec![2, 3])]
    fn write_carray_round_trip(#[case] array: Vec<u32>, #[case] chunk_shape: Vec<usize>)
    {
        let array            = Array::from_shape_vec(chunk_shape.clone(), array).unwrap();
        let (_dir, filename) = tempfile("write_carray_round_trip");
        let file             = hdf5::File::create(&filename).unwrap();
        write_chunked_array(Rc::new(file), "/here", chunk_shape.clone(), &array).unwrap();

        let read = read_array::<u32>(&filename, "/here").unwrap();
        assert_eq!(read.shape(), chunk_shape);

    }

}
