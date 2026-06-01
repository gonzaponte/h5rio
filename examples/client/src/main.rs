use std::io::Result;
use std::rc::Rc;

use ndarray::{arr2, s};
use h5rio::{h5type, ArrayHdf5Writer, TableHdf5Writer, read_array, read_table};
use hdf5_metno as hdf5;

#[h5type]
struct Hit {
    event_id: u64,
    sensor_id: u32,
    charge: f32,
}


fn main() -> Result<()> {
    let filename = "/tmp/example.h5";
    let file = Rc::new(hdf5::File::create(filename)?);

    let table_writer = TableHdf5Writer::<Hit>::new(Rc::clone(&file), "/hits" , 1)?;
    let array_writer = ArrayHdf5Writer::<f32>::new(Rc::clone(&file), "/waves", 1, vec![2, 3])?;

    table_writer.write(Hit {
        event_id: 0,
        sensor_id: 12,
        charge: 18.4,
    })?;

    array_writer.write(arr2(&[ [0.1, 1.2, 2.3]
                             , [3.4, 4.5, 5.6]]))?;

    // calling flush directly is also OK
    // The chunk_size=1 above also ensures that
    drop(table_writer);
    drop(array_writer);

    let table = read_table::<Hit>(filename, "/hits" )?;
    let array = read_array::<f32>(filename, "/waves")?;

    println!("Read {} entries from table", table.len());
    println!("1st table entry: {:?}"     , table[0]);

    println!("Read an array of shape {:?}", array.shape());
    println!("1st array entry: \n{:?}"          , array.slice(s![0, .., ..]));

    Ok(())
}
