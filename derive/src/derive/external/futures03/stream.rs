use crate::utils::*;

pub(crate) const NAME: &[&str] = &["futures::Stream"];

pub(crate) fn derive(data: &Data, items: &mut Vec<ItemImpl>) -> Result<()> {
    derive_trait!(
        data,
        parse_quote!(::futures::stream::Stream)?,
        parse_quote! {
            trait Stream {
                type Item;
                fn poll_next(
                    self: ::core::pin::Pin<&mut Self>,
                    cx: &mut ::core::task::Context<'_>,
                ) -> ::core::task::Poll<::core::option::Option<Self::Item>>;
                fn size_hint(&self) -> (usize, ::core::option::Option<usize>);
            }
        }?,
    )
    .map(|item| items.push(item))
}
