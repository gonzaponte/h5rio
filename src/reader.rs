use std::marker::PhantomData;

use hdf5_metno as hdf5;

use ndarray::{s, ArrayD, IxDyn};


pub fn read_table<T: hdf5::H5Type>(filename: &str, dataset : &str) -> hdf5::Result<Vec<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    dataset.read_slice_1d::<T,_>(s![..]).map(|v| v.into_raw_vec_and_offset().0)
}

#[derive(Debug)]
pub struct Hdf5TableIter<T> {
    _file: hdf5::File, // keep file alive
    dataset: hdf5::Dataset,
    index: usize,
    len: usize,
    _inner_type: PhantomData<T>,
}

pub fn iter_table<T: hdf5::H5Type>(filename: &str, dataset: &str) -> hdf5::Result<Hdf5TableIter<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    let shape   = dataset.shape();

    if shape.len() != 1 {
        let msg = format!("iter_table: expected a one-dimensional table dataset, got shape {shape:?}");
        return Err(hdf5::Error::Internal(msg));
    }

    Ok(Hdf5TableIter{_file: file, dataset, index:0, len: shape[0], _inner_type: PhantomData})
}

impl<T> Iterator for Hdf5TableIter<T> where T: hdf5::H5Type {
    type Item = hdf5::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }

        let entry = self.dataset
                        .read_slice_1d::<T,_>(s![self.index..self.index + 1])
                        .and_then(|data| data.into_iter()
                                             .next()
                                             .ok_or_else(|| {
                                                 let msg = "iter_table: one-row slice returned no rows".to_owned();
                                                 hdf5::Error::Internal(msg)
                                             }));
        self.index += 1;
        Some(entry)
    }
}

pub fn read_array<T: hdf5::H5Type>(filename: &str, dataset : &str) -> hdf5::Result<ArrayD<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    dataset.read_dyn::<T>()
        .and_then(|data| {
            let read_shape = data.shape().to_vec();
            data.into_shape_with_order(dataset.shape())
                .map_err(|error| {
                    let msg = format!("could not reshape array of shape {:?}\
                                       into {:?}.\n{}",
                                      read_shape, dataset.shape(), error);
                    hdf5::Error::Internal(msg)
                })
           })
}

#[derive(Debug)]
pub struct Hdf5ArrayIter<T> {
    _file: hdf5::File, // keep file alive
    dataset: hdf5::Dataset,
    index: usize,
    len: usize,
    ndim: usize,
    _inner_type: PhantomData<T>,
}

pub fn iter_array<T: hdf5::H5Type>(filename: &str, dataset: &str) -> hdf5::Result<Hdf5ArrayIter<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    let shape   = dataset.shape();

    if shape.is_empty() {
        let msg = "iter_array: cannot iterate over a scalar dataset".to_owned();
        return Err(hdf5::Error::Internal(msg));
    }

    Ok(Hdf5ArrayIter{_file: file, dataset, index:0, len: shape[0], ndim: shape.len(), _inner_type: PhantomData})
}


impl<T> Iterator for Hdf5ArrayIter<T> where T: hdf5::H5Type {
    type Item = hdf5::Result<ArrayD<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }

        let mut slice = Vec::with_capacity(self.ndim);
        slice.push(hdf5::SliceOrIndex::Index(self.index));
        for _ in 1..self.ndim {
            slice.push(hdf5::SliceOrIndex::Unlimited{start:0, step:1, block:1})
        }
        let slice = hdf5::Hyperslab::from(slice);
        let entry = self.dataset.read_slice::<T, _, IxDyn>(slice);
        self.index += 1;
        return Some(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use pretty_assertions::assert_eq;

    use h5rio_macros::h5type;

    #[h5type]
    pub struct DummyData {
        i: u32,
        x: f32,
    }

    #[allow(dead_code)]
    fn setup_file() {
        // This is the file read by the test. It is not produced on every run,
        // but it is available to update the test file if necessary.
        use std::rc::Rc;
        use hdf5_metno as hdf5;
        use ndarray::arr2;
        use crate::TableHdf5Writer;
        use crate::ArrayHdf5Writer;

        let file = "data/table_and_array.h5";
        let file = hdf5::File::create(file).unwrap();
        let file = Rc::new(file);
        let w    = TableHdf5Writer::new(Rc::clone(&file), "/group/table", 3).unwrap();

        w.write(DummyData{i:  5, x: 1.62}).unwrap();
        w.write(DummyData{i:  1, x: 2.72}).unwrap();
        w.write(DummyData{i: 42, x: 3.14}).unwrap();
        drop(w);

        let w = ArrayHdf5Writer::new(Rc::clone(&file), "/group/array", 3, vec![2,3]).unwrap();

        w.write(arr2( &[[ 1.,  2.,  3. ], [ 4.,  5.,  6.]] )).unwrap();
        w.write(arr2( &[[11., 22., 33. ], [44., 55., 66.]] )).unwrap();
        drop(w);
    }

    #[test]
    fn read_table_custom() {
        let read = read_table::<DummyData>("data/table_and_array.h5", "/group/table").unwrap();
        assert_eq!(read[0].i,  5);
        assert_eq!(read[1].i,  1);
        assert_eq!(read[2].i, 42);

        assert_float_eq!(read[0].x, 1.62, ulps<=2);
        assert_float_eq!(read[1].x, 2.72, ulps<=2);
        assert_float_eq!(read[2].x, 3.14, ulps<=2);
    }

    #[test]
    fn iter_table_custom() {
        let mut iter = iter_table::<DummyData>("data/table_and_array.h5", "/group/table").unwrap();

        let first  = iter.next().unwrap().unwrap();
        let second = iter.next().unwrap().unwrap();
        let third  = iter.next().unwrap().unwrap();
        assert!(iter.next().is_none());

        assert_eq!( first.i,  5);
        assert_eq!(second.i,  1);
        assert_eq!( third.i, 42);

        assert_float_eq!( first.x, 1.62, ulps<=2);
        assert_float_eq!(second.x, 2.72, ulps<=2);
        assert_float_eq!( third.x, 3.14, ulps<=2);
    }

    #[test]
    fn iter_table_rejects_multidimensional_dataset() {
        let (_dir, filename) = crate::utils::tempfile("iter_table_rejects_multidimensional_dataset");
        let file             = hdf5::File::create(&filename).unwrap();
        file.new_dataset::<i32>()
            .shape([2, 2])
            .create("/array")
            .unwrap();

        let out = iter_table::<i32>(&filename, "/array");

        assert!(matches!(out, Err(hdf5::Error::Internal(_))));
        assert!(out.unwrap_err().to_string().contains("expected a one-dimensional table dataset"));
    }

    #[test]
    fn read_array_custom() {
        let read = read_array::<f64>("data/table_and_array.h5", "/group/array").unwrap();

        assert_float_eq!(read[[0,0,0]],  1.0, ulps<=2);
        assert_float_eq!(read[[0,0,1]],  2.0, ulps<=2);
        assert_float_eq!(read[[0,0,2]],  3.0, ulps<=2);
        assert_float_eq!(read[[0,1,0]],  4.0, ulps<=2);
        assert_float_eq!(read[[0,1,1]],  5.0, ulps<=2);
        assert_float_eq!(read[[0,1,2]],  6.0, ulps<=2);
        assert_float_eq!(read[[1,0,0]], 11.0, ulps<=2);
        assert_float_eq!(read[[1,0,1]], 22.0, ulps<=2);
        assert_float_eq!(read[[1,0,2]], 33.0, ulps<=2);
        assert_float_eq!(read[[1,1,0]], 44.0, ulps<=2);
        assert_float_eq!(read[[1,1,1]], 55.0, ulps<=2);
        assert_float_eq!(read[[1,1,2]], 66.0, ulps<=2);
    }

    #[test]
    fn iter_array_custom() {
        let mut iter = iter_array::<f64>("data/table_and_array.h5", "/group/array").unwrap();

        let first = iter.next().unwrap().unwrap();
        assert_eq!(first.shape(), &[2, 3]);
        assert_float_eq!(first[[0,0]], 1.0, ulps<=2);
        assert_float_eq!(first[[0,1]], 2.0, ulps<=2);
        assert_float_eq!(first[[0,2]], 3.0, ulps<=2);
        assert_float_eq!(first[[1,0]], 4.0, ulps<=2);
        assert_float_eq!(first[[1,1]], 5.0, ulps<=2);
        assert_float_eq!(first[[1,2]], 6.0, ulps<=2);

        let second = iter.next().unwrap().unwrap();
        assert_eq!(second.shape(), &[2, 3]);
        assert_float_eq!(second[[0,0]], 11.0, ulps<=2);
        assert_float_eq!(second[[0,1]], 22.0, ulps<=2);
        assert_float_eq!(second[[0,2]], 33.0, ulps<=2);
        assert_float_eq!(second[[1,0]], 44.0, ulps<=2);
        assert_float_eq!(second[[1,1]], 55.0, ulps<=2);
        assert_float_eq!(second[[1,2]], 66.0, ulps<=2);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_array_rejects_scalar_dataset() {
        let (_dir, filename) = crate::utils::tempfile("iter_array_rejects_scalar_dataset");
        let file             = hdf5::File::create(&filename).unwrap();
        file.new_dataset::<i32>()
            .shape(())
            .create("/scalar")
            .unwrap()
            .write_scalar(&42)
            .unwrap();

        let out = iter_array::<i32>(&filename, "/scalar");

        assert!(matches!(out, Err(hdf5::Error::Internal(_))));
        assert!(out.unwrap_err().to_string().contains("cannot iterate over a scalar dataset"));
    }
}
