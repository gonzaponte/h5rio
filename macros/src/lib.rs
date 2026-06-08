use proc_macro::TokenStream;
use quote::quote;

/// Derive the traits and representation needed for HDF5 table row structs.
///
/// The macro expands the annotated struct with:
///
/// ```ignore
/// #[derive(::hdf5_metno::H5Type, Clone, PartialEq, Debug)]
/// #[repr(C)]
/// ```
///
/// It is intended for plain record structs used with `TableHdf5Writer` and
/// `read_table`.
///
/// The struct fields must support the generated derives, including
/// `hdf5_metno::H5Type`. The macro does not inspect fields, rewrite schemas, or
/// provide migration between incompatible row definitions. Changing field
/// names, field order, or field types changes the HDF5 compound type expected
/// by readers and writers.
///
/// Avoid duplicating the generated derives on the same struct, because that
/// would create duplicate trait implementations.
#[proc_macro_attribute]
pub fn h5type(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);

    quote! {
        #[derive(::hdf5_metno::H5Type, Clone, PartialEq, Debug)]
        #[repr(C)]
        #item
    }
    .into()
}
