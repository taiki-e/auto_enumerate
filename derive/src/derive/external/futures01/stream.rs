use crate::utils::*;

pub(crate) const NAME: &[&str] = &["futures01::Stream"];

pub(crate) fn derive(data: &Data, items: &mut Vec<ItemImpl>) -> Result<()> {
    let crate_ = quote!(::futures);

    derive_trait!(
        data,
        parse_quote!(#crate_::stream::Stream)?,
        parse_quote! {
            trait Stream {
                type Item;
                type Error;
                fn poll(&mut self) -> #crate_::Poll<::core::option::Option<Self::Item>, Self::Error>;
            }
        }?,
    )
    .map(|item| items.push(item))
}
