extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote};

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
pub fn rari_f(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(input as syn::ItemFn);

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
    proc_macro::TokenStream::from(quote!(
        #[allow(dead_code)]
        #function
        #dup
    ))
}
