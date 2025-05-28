extern crate proc_macro;
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::fmt::Write;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Lit};

fn poor_ts_fn_outline_conversion(outline: &str, complete: bool) -> Option<String> {
    let outline = outline.replace("\n", " ");
    let outline = outline.strip_prefix("fn ")?;
    let args_start = outline.find('(')?;
    let args_end = outline.find(")")?;

    let mut out = String::with_capacity(outline.len());
    out.push_str(&outline[..args_start]);
    out.push('(');
    let mut first = true;
    for (i, arg) in outline[args_start + 1..args_end].split(",").enumerate() {
        if arg.is_empty() {
            break;
        }
        if first {
            first = false;
        } else {
            out.push_str(", ");
        }
        let arg = arg.trim();
        let colon = arg.find(" : ")?;
        let typ = &arg[colon + 3..];
        if complete {
            write!(&mut out, "${{{}:{}}}", i + 1, &arg[..colon]).unwrap();
        } else if let Some(optional_type) = typ.strip_prefix("Option < ") {
            out.extend([&arg[..colon], "?: ", optional_type.strip_suffix(" >")?]);
        } else {
            out.extend([&arg[..colon], ": ", &arg[colon + 3..]]);
        }
    }
    out.push(')');
    Some(out)
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

    let outline_string = function.sig.to_token_stream().to_string();
    let outline =
        &poor_ts_fn_outline_conversion(&outline_string, false).unwrap_or(outline_string.clone());
    let complete = &poor_ts_fn_outline_conversion(&outline_string, true).unwrap_or(name.clone());

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
                    outline: #outline,
                    complete: #complete,
                    doc: #doc_string,
                    function: #dup_ident,
                    is_sidebar: #is_sidebar,
                }
            }
        }
    } else {
        quote! {}
    };

    proc_macro::TokenStream::from(quote!(
        #collect
        #[allow(dead_code)]
        #function
        #dup
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ts_conversion() {
        let outline = "fn\nhttpheader(status : AnyArg, display : Option < String > , anchor : Option <\nString > , no_code : Option < AnyArg > ,) -> Result < String, DocError >";
        println!("{:?}", poor_ts_fn_outline_conversion(outline, true));
        println!("{:?}", poor_ts_fn_outline_conversion(outline, false));
    }
}
