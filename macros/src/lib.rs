use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn h5type(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);

    quote! {
        #[derive(hdf5::H5Type, Clone, PartialEq, Debug)]
        #[repr(C)]
        #item
    }
    .into()
}
