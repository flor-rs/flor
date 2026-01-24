use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::Ident;

pub(crate) fn flor_crate() -> Ident {
    match crate_name("flor") {
        Ok(flor) => match flor {
            FoundCrate::Itself => Ident::new("crate", Span::call_site()),
            FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
        },
        Err(_) => {
            Ident::new("flor", Span::call_site())
        }
    }
}