extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::cell::RefCell;
use syn::__private::Span;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, LitBool, LitStr, Macro, Result, Token,
};

thread_local! {
    static STATE: RefCell<Vec<&'static str>> = Default::default();
}

#[derive(Clone)]
enum FieldIdentities {
    StructName(Span, Option<Ident>),
    Path(Span, Option<(String, Span)>),
    SPV(Span, Option<(bool, Span)>),
    Src(Span, Option<(String, Span)>),
    Lang(Span, Option<(String, Span)>),
    Compress(Span, Option<(bool, Span)>),
    Bytes(Span, Option<Macro>),
}

impl Parse for FieldIdentities {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();
        let ident = input.parse::<Ident>()?.to_string();
        let (field_name, mut ident) = match ident.as_str() {
            "struct_name" => ("struct_name", FieldIdentities::StructName(span, None)),
            "path" => ("path", FieldIdentities::Path(span, None)),
            "spv" => ("spv", FieldIdentities::SPV(span, None)),
            "src" => ("src", FieldIdentities::Src(span, None)),
            "lang" => ("lang", FieldIdentities::Lang(span, None)),
            "compress" => ("compress", FieldIdentities::Compress(span, None)),
            "bytes" => ("bytes", FieldIdentities::Bytes(span, None)),
            _ => {
                return Err(syn::Error::new(
                    span,
                    format!("Unexpected field: {}", ident),
                ));
            }
        };

        if let Some(e) = STATE.with_borrow_mut(|state| {
            if state.contains(&field_name) {
                Some(syn::Error::new(span, "Duplicate field"))
            } else {
                state.push(field_name);
                None
            }
        }) {
            return Err(e);
        }

        input.parse::<Token![:]>()?;

        match &mut ident {
            FieldIdentities::StructName(_, ref mut o) => {
                let _ = o.insert(input.parse()?);
            }
            FieldIdentities::Path(_, ref mut o) => {
                let parse = input.parse::<LitStr>()?;
                let value = parse.value();
                let span = parse.span();
                let _ = o.insert((value, span));
            }
            FieldIdentities::SPV(_, ref mut o) => {
                let parse = input.parse::<LitBool>()?;
                let value = parse.value;
                let span = parse.span();
                let _ = o.insert((value, span));
            }
            FieldIdentities::Src(_, ref mut o) => {
                let parse = input.parse::<LitStr>()?;
                let value = parse.value();
                let span = parse.span();
                let _ = o.insert((value, span));
            }
            FieldIdentities::Lang(_, ref mut o) => {
                let parse = input.parse::<LitStr>()?;
                let value = parse.value();
                let span = parse.span();
                let _ = o.insert((value, span));
            }
            FieldIdentities::Compress(_, ref mut o) => {
                let parse = input.parse::<LitBool>()?;
                let value = parse.value;
                let span = parse.span();
                let _ = o.insert((value, span));
            }
            FieldIdentities::Bytes(_, ref mut o) => {
                let _ = o.insert(input.parse()?);
            }
        }

        Ok(ident)
    }
}

struct Args {
    fields: Punctuated<FieldIdentities, token::Comma>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _ = braced!(content in input);
        Ok(Args {
            fields: content.parse_terminated(FieldIdentities::parse, token::Comma)?,
        })
    }
}

#[proc_macro]
pub fn include_shader(input: TokenStream) -> TokenStream {
    let args: Args = match syn::parse(input) {
        Ok(a) => a,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    process_args(args)
}

enum Source {
    FromSource(String),
    FromSourceFile(String),
    FromBytes(&'static [u8]),
    FromCompiledFile(String),
}

fn process_args(args: Args) -> TokenStream {
    quote! {}.into()
}
