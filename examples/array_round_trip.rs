use std::rc::Rc;

use ndarray::{arr2,s};
use h5rio::{ArrayHdf5Writer, read_array};
use hdf5_metno as hdf5;


fn main() -> hdf5::Result<()> {
    let filename = "/tmp/example_array.h5";
    let file = Rc::new(hdf5::File::create(filename)?);

    let writer = ArrayHdf5Writer::<f32>::new(file, "/waves", 2, vec![2, 3])?;

    writer.write(arr2(&[ [0.1, 1.2, 2.3]
                       , [3.4, 4.5, 5.6]]))?;

    writer.write(arr2(&[ [-0.1, -1.2, -2.3]
                       , [-3.4, -4.5, -5.6]]))?;

    // dropping the writer would also trigger a flush
    // the used chunk size of 2 above has also triggered a flush
    writer.flush()?;

    let data = read_array::<f32>(filename, "/waves")?;

    println!("Read an array of shape {:?}", data.shape());
    println!("1st entry: \n{:?}", data.slice(s![0, .., ..]));
    println!("2nd entry: \n{:?}", data.slice(s![1, .., ..]));

    Ok(())
}
