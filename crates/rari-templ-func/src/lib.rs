extern crate proc_macro;
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Lit};

fn pascal_case(s: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    for ch in s.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

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

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct RariFargs {
    #[darling(default)]
    register: Option<syn::TypePath>,
    #[darling(default)]
    sidebar: bool,
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
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let attr_args = match RariFargs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    //let attr_args = RariFargs::default();
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

    function
        .sig
        .inputs
        .insert(0, parse_quote!(env: &::rari_types::RariEnv));

    let f_struct = format_ident!("{}", pascal_case(&name));
    let r_type = match function.sig.output {
        syn::ReturnType::Default => parse_quote!(()),
        syn::ReturnType::Type(_, ref t) => t.clone(),
    };
    let dup_ident = dup.sig.ident.clone();
    let is_sidebar: Lit = if attr_args.sidebar {
        parse_quote!(true)
    } else {
        parse_quote!(false)
    };
    let collect = if let Some(inventory_type) = attr_args.register {
        quote! {
            inventory::submit! {
                #inventory_type {
                    name: #name,
                    outline: #outline_static_ident,
                    doc: #doc_static_ident,
                    function: #dup_ident,
                    is_sidebar: #is_sidebar,
                }
            }
        }
    } else {
        quote! {}
    };

    proc_macro::TokenStream::from(quote!(
        #doc
        #outline
        #collect
        #[allow(dead_code)]
        #function
        #dup

        pub struct #f_struct {}

        impl ::rari_types::templ::RariF for #f_struct {
            type R = #r_type;

            fn function() -> ::rari_types::templ::RariFn<Self::R> {
                #dup_ident
            }
            fn doc() -> &'static str {
                #doc_static_ident
            }
            fn outline() -> &'static str {
                #outline_static_ident
            }
            fn is_sidebar() -> bool {
                #is_sidebar
            }
        }
    ))
}
