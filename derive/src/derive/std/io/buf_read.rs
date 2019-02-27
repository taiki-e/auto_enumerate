use proc_macro2::TokenStream;

use crate::utils::*;

pub(crate) const NAME: &[&str] = &["BufRead", "io::BufRead"];

pub(crate) fn derive(data: &Data) -> Result<TokenStream> {
    let io = quote!(::std::io);

    derive_trait!(
        data,
        parse_quote!(#io::BufRead)?,
        parse_quote! {
            trait BufRead {
                #[inline]
                fn fill_buf(&mut self) -> #io::Result<&[u8]>;
                #[inline]
                fn consume(&mut self, amt: usize);
                #[inline]
                fn read_until(&mut self, byte: u8, buf: &mut ::std::vec::Vec<u8>) -> #io::Result<usize>;
                #[inline]
                fn read_line(&mut self, buf: &mut ::std::string::String) -> #io::Result<usize>;
            }
        }?,
    )
}
