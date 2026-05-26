use h5rio_macros::h5type;
use hdf5_metno as hdf5;

#[h5type]
struct Dummy {
    a: u16,
    b: f32,
    c: i64,
    d: f64,
}


#[test]
fn generated_derives_work() {
    let a = Dummy {
        a: 42,
        b: 2.72,
        c: -99,
        d: 3.14159,
    };

    let b = a.clone();

    assert_eq!(a, b);
    assert_eq!(format!("{a:?}"), "Dummy { a: 42, b: 2.72, c: -99, d: 3.14159 }");
}


#[test]
fn generated_hdf5_type_impl_exists() {
    fn requires_h5_type<T: hdf5::H5Type>() {}

    requires_h5_type::<Dummy>();
}


#[test]
fn generated_layout_matches_expected_c_layout() {
    use std::mem::{align_of, offset_of, size_of};

    // With #[repr(C)], fields remain in declaration order and padding is
    // inserted so each field begins at an offset satisfying its alignment:
    // `a` starts at 0; after its 2 bytes, 2 padding bytes are inserted so
    // `b` starts at 4; `c` and `d` then naturally start at offsets 8 and 16.
    //
    // The struct alignment is 8 bytes, because its most-aligned fields,
    // `c: i64` and `d: f64`, each require 8-byte alignment.
    //
    // The total size is 24 bytes: 2 bytes for `a`, 2 bytes of padding,
    // 4 bytes for `b`, 8 bytes for `c`, and 8 bytes for `d`. No trailing
    // padding is needed because 24 is already a multiple of 8.
    assert_eq!(offset_of!(Dummy, a), 0      );
    assert_eq!(offset_of!(Dummy, b), 4      );
    assert_eq!(offset_of!(Dummy, c), 4+4    );
    assert_eq!(offset_of!(Dummy, d), 4+4+8  );
    assert_eq!(align_of::<Dummy>() ,       8);
    assert_eq!( size_of::<Dummy>() , 4+4+8+8);
}
