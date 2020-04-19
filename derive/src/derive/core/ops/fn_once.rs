use crate::utils::*;

pub(crate) const NAME: &[&str] = &["FnOnce"];

pub(crate) fn derive(data: &Data, items: &mut Vec<ItemImpl>) -> Result<()> {
    let trait_path = quote!(::core::ops::FnOnce);
    let trait_ = quote!(#trait_path(__T) -> __U);
    let fst = data.fields().iter().next();

    let mut impl_ = data.impl_with_capacity(2)?;

    *impl_.trait_() =
        Some(Trait::new(syn::parse2(trait_path.clone())?, parse_quote!(#trait_path<(__T,)>)?));
    impl_.push_generic_param(param_ident!("__T"));
    impl_.push_generic_param(param_ident!("__U"));

    impl_.push_where_predicate(parse_quote!(#fst: #trait_)?);
    data.fields()
        .iter()
        .skip(1)
        .try_for_each(|f| parse_quote!(#f: #trait_).map(|f| impl_.push_where_predicate(f)))?;

    impl_.append_items_from_trait(parse_quote! {
        trait FnOnce {
            type Output;
            extern "rust-call" fn call_once(self, args: (__T,)) -> Self::Output;
        }
    }?)?;

    items.push(impl_.build_item());
    Ok(())
}
