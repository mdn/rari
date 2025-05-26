extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Lit};

fn is_option(path: &syn::TypePath) -> bool {
    let idents_of_path = path.path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push(':');
        acc
    });
    vec!["Option:", "std:option:Option:", "core:option:Option:"]
        .into_iter()
        .any(|s| idents_of_path == *s)
}

/// Define rari templ functions.
///
/// Example:
/// ```
/// use rari_templ_func::rari_f;
///
/// #[rari_f]
/// fn hello(a: String) -> Result<String, anyhow::Error> {
///     Ok(format!("Hello {}!", a))
/// }
/// ```
/// This will automatically inject an argument `env` providing a
/// [RariEnv] reference.
#[proc_macro_attribute]
pub fn rari_f(attr: TokenStream, input: TokenStream) -> TokenStream {
    let inventory_type = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as syn::TypePath))
    };
    // Parse the input as a function.
    let mut function = parse_macro_input!(input as syn::ItemFn);

    // Extract doc comments from attributes.
    let doc_comments: Vec<String> = function
        .attrs
        .iter()
        .filter(|attr| matches!(attr.style, syn::AttrStyle::Outer))
        .filter_map(|attr| {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(lit) = &meta.value {
                    if let Lit::Str(value) = &lit.lit {
                        Some(value.value())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    let doc_string = doc_comments.join("\n");
    let name = function.sig.ident.to_string();
    let doc_static_ident = format_ident!("DOC_FOR_{}", name.to_uppercase());
    let outline_static_ident = format_ident!("OUTLINE_FOR_{}", name.to_uppercase());

    let doc = quote! {
        pub static #doc_static_ident: &str = #doc_string;
    };
    let original_sig = &function.sig.to_token_stream().to_string();
    let outline = quote! {
        pub static #outline_static_ident: &str = #original_sig;
    };

    let mut dup = function.clone();
    dup.sig.ident = format_ident!("{}_{}", dup.sig.ident, "any");
    let args = dup.sig.inputs;
    dup.sig.inputs = Punctuated::new();
    dup.sig
        .inputs
        .push(parse_quote!(env: &::rari_types::RariEnv));
    dup.sig
        .inputs
        .push(parse_quote!(args: Vec<Option<::rari_types::Arg>>));
    let mapping = args.iter().filter_map(|arg| match arg {
		syn::FnArg::Typed(ty) => Some(ty),
		_ => None,
	}).map(|arg| {
	let ty = &arg.ty;
	let option = if let syn::Type::Path(p) = &**ty {
		is_option(p)
	} else {
		false
	};
	let ident = &arg.pat;
	if option {
		quote! { let #ident: #ty = match args_iter.next().map(|arg| arg.map(TryInto::try_into)) {
            None => None,
			Some(None) => None,
            Some(Some(Ok(o))) => Some(o),
            Some(Some(Err(e))) => {
              return Err(e.into());
            },
		}; }
	} else {
		quote! { let #ident: #ty = args_iter.next().flatten().ok_or(ArgError::MustBeProvided).and_then(TryInto::try_into)?; }
	}
	});
    let block = dup.block;
    dup.block = parse_quote! {
        {
            use ::rari_types::ArgError;
            use std::convert::TryInto;
            let mut args_iter = args.into_iter();
            #(#mapping)*
            #block
        }
    };

    let collect = if let Some(inventory_type) = inventory_type {
        quote! {
            inventory::submit! {
                #inventory_type { name: #name, outline: #outline_static_ident, doc: #doc_static_ident }
            }
        }
    } else {
        quote! {}
    };

    function
        .sig
        .inputs
        .insert(0, parse_quote!(env: &::rari_types::RariEnv));
    proc_macro::TokenStream::from(quote!(
        #doc
        #outline
        #collect
        #[allow(dead_code)]
        #function
        #dup
    ))
}
