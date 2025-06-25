use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{Arg, RariEnv};

pub type RariFn<R> = fn(&RariEnv<'_>, Vec<Option<Arg>>) -> R;

#[derive(Debug, Default, FromMeta, Clone, Copy, strum::Display)]
pub enum TemplType {
    #[default]
    None,
    Link,
    Sidebar,
    Banner,
}

impl ToTokens for TemplType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = match self {
            TemplType::None => quote! { TemplType::None },
            TemplType::Link => quote! { TemplType::Link },
            TemplType::Sidebar => quote! { TemplType::Sidebar },
            TemplType::Banner => quote! { TemplType::Banner },
        };
        tokens.extend(variant);
    }
}
