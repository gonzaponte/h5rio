use hdf5_metno as hdf5;

use ndarray::{s, ArrayD};


pub fn read_table<T: hdf5::H5Type>(filename: &str, dataset : &str) -> hdf5::Result<Vec<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    dataset.read_slice_1d::<T,_>(s![..]).map(|v| v.into_raw_vec_and_offset().0)
}

pub fn read_array<T: hdf5::H5Type + Clone>(filename: &str, dataset : &str) -> hdf5::Result<ArrayD<T>> {
    let file    = hdf5::File::open(filename)?;
    let dataset = file.dataset(dataset)?;
    dataset.read_dyn::<T>()
           .map(|data| data.into_shape_with_order(dataset.shape())
                           .expect("Could not reshape read array") )
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
}
