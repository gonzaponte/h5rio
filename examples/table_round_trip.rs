use std::io::Result;
use std::rc::Rc;

use h5rio::{h5type, TableHdf5Writer, read_table};
use hdf5_metno as hdf5;


#[h5type]
struct Hit {
    event_id: u64,
    sensor_id: u32,
    charge: f32,
}


fn main() -> Result<()> {
    let filename = "/tmp/example_table.h5";
    let file = Rc::new(hdf5::File::create(filename)?);

    let writer = TableHdf5Writer::<Hit>::new(file, "/hits", 2)?;

    writer.write(Hit {
        event_id: 0,
        sensor_id: 12,
        charge: 18.4,
    })?;

    writer.write(Hit {
        event_id: 9,
        sensor_id: 42,
        charge: 1.44,
    })?;

    // dropping the writer would also trigger a flush
    // the used chunk size of 2 above has also triggered a flush
    writer.flush()?;

    let data = read_table::<Hit>(filename, "/hits")?;
    // OR
    // let data : Vec<Hit> = read_table(filename, "/hits")?;

    println!("Read {} entries", data.len());
    println!("1st entry: {:?}", data[0]);
    println!("2nd entry: {:?}", data[1]);

    Ok(())
}
