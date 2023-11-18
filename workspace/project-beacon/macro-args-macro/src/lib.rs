use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse,
    punctuated::Punctuated,
    token::{self, Brace, Bracket},
    LitByteStr, LitStr, Path, Result,
};

#[proc_macro]
pub fn parseable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse::<MacroStructure>(input)
        .and_then(MacroStructure::into_tokens)
        .map_err(syn::Error::into_compile_error)
        .map_or_else(Into::into, Into::into)
}

struct MacroStructure {
    structs: Vec<Struct>,
    root_struct: Ident,
}

impl Parse for MacroStructure {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut structs = Vec::new();
        let root_struct = loop {
            if input.peek2(token::Brace) {
                structs.push(input.parse::<Struct>()?);
            } else {
                let rs = input.parse::<Ident>()?;
                if !input.is_empty() {
                    return Err(input.error("Unexpected token!"));
                }
                break rs;
            }
        };

        Ok(Self {
            structs,
            root_struct,
        })
    }
}

impl ToTokens for MacroStructure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for s in &self.structs {
            s.to_tokens(tokens);
        }
        self.root_struct.to_tokens(tokens);
    }
}

impl MacroStructure {
    fn into_tokens(self) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();
        for s in self.structs {
            s.into_tokens()?.to_tokens(&mut tokens);
        }
        Ok(tokens)
    }
}

struct Struct {
    ident: Ident,
    brace: Brace,
    fields: Punctuated<StructFieldSection, token::Comma>,
}

impl Parse for Struct {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let content;
        let brace = syn::braced!(content in input);
        let fields = content.parse_terminated(StructFieldSection::parse, token::Comma)?;
        Ok(Self {
            ident,
            brace,
            fields,
        })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.brace.surround(tokens, |t| self.fields.to_tokens(t));
    }
}

impl Struct {
    fn into_tokens(self) -> Result<TokenStream> {
        let mut struct_fields_defs = TokenStream::new();
        let mut group_tracks = TokenStream::new();
        let mut field_tracks_type = TokenStream::new();
        let mut field_tracks_value = TokenStream::new();
        let mut match_branches = TokenStream::new();
        let mut non_optional_handling = TokenStream::new();
        let mut struct_constructor_fields = TokenStream::new();
        let field_tracker_ident = format_ident!("field_tracker");
        let parsed_ident_ident = format_ident!("ident");
        let field_name_ident = format_ident!("field_name");
        let parse_stream_ident = format_ident!("content");
        let mut field_index = 0;
        for (i, f) in self.fields.iter().enumerate() {
            if i > 0 {
                quote! {,}.to_tokens(&mut struct_fields_defs);
                quote! {,}.to_tokens(&mut field_tracks_type);
                quote! {,}.to_tokens(&mut field_tracks_value);
                quote! {,}.to_tokens(&mut struct_constructor_fields);
            }

            f.contribute_tokens(
                &mut struct_fields_defs,
                &mut group_tracks,
                i,
                &mut field_tracks_type,
                &mut field_tracks_value,
                &field_tracker_ident,
                &mut field_index,
                &mut match_branches,
                &parsed_ident_ident,
                &parse_stream_ident,
                &mut non_optional_handling,
                &mut struct_constructor_fields,
            );
        }
        let ident = &self.ident;
        Ok(quote! {
            struct #ident { #struct_fields_defs }
            impl ::syn::parse::Parse for #ident {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                    #group_tracks
                    let mut field_tracker: (#field_tracks_type) = (#field_tracks_value);
                    let brace = syn::__private::parse_braces(&input)?;
                    let #parse_stream_ident = brace.content;
                    // let brace = brace.token;
                    while !#parse_stream_ident.is_empty() {
                        let #parsed_ident_ident = content.parse::<::syn::Ident>()?;
                        let #field_name_ident = #parsed_ident_ident.to_string();
                        match #field_name_ident.as_str() {
                            #match_branches
                            _ => {
                                return ::std::result::Result::Err(
                                    ::syn::Error::new_spanned(
                                        #parsed_ident_ident,
                                        ::std::format!("Unexpected field: {}", #field_name_ident)
                                    )
                                );
                            }
                        }
                        if #parse_stream_ident.is_empty() {
                            break;
                        }
                        #parse_stream_ident.parse::<::syn::token::Comma>()?;
                    }
                    #non_optional_handling
                    Ok(Self{ #struct_constructor_fields })
                }
            }
        }
        .into_token_stream())
    }
}

enum StructFieldSection {
    Group(
        token::Pound,
        Bracket,
        Ident,
        Brace,
        Punctuated<Field, token::Comma>,
    ),
    Field(Field),
}

impl Parse for StructFieldSection {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(token::Pound) {
            let pound = input.parse::<token::Pound>()?;
            let content;
            let bracket = syn::bracketed!(content in input);
            let ident = content.parse::<Ident>()?;
            if !content.is_empty() {
                return Err(content.error("Unexpected token!"));
            }
            let content;
            let brace = syn::braced!(content in input);
            let punct = content.parse_terminated(Field::parse, token::Comma)?;
            Ok(Self::Group(pound, bracket, ident, brace, punct))
        } else {
            Ok(Self::Field(input.parse()?))
        }
    }
}

impl ToTokens for StructFieldSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Field(f) => {
                f.to_tokens(tokens);
            }
            Self::Group(pound, brack, ident, brace, punct) => {
                pound.to_tokens(tokens);
                brack.surround(tokens, |t| ident.to_tokens(t));
                brace.surround(tokens, |t| punct.to_tokens(t));
            }
        }
    }
}

impl StructFieldSection {
    fn contribute_tokens(
        &self,
        struct_fields_defs: &mut TokenStream,
        group_tracks: &mut TokenStream,
        group_id: usize,
        field_tracks_type: &mut TokenStream,
        field_tracks_value: &mut TokenStream,
        field_tracker_ident: &Ident,
        field_index: &mut usize,
        match_branches: &mut TokenStream,
        parsed_ident_ident: &Ident,
        parse_stream_ident: &Ident,
        non_optional_handling: &mut TokenStream,
        struct_constructor_fields: &mut TokenStream,
    ) {
        let (group, group_field_names) = match self {
            Self::Field(f) => (vec![f], None),
            Self::Group(_, _, _, _, punc) => (
                punc.iter().collect(),
                Some(punc.iter().map(|f| f.ident.to_string()).collect::<Vec<_>>()),
            ),
        };
        for (i, f) in group.iter().enumerate() {
            if i > 0 {
                quote! {,}.to_tokens(struct_fields_defs);
                quote! {,}.to_tokens(field_tracks_type);
                quote! {,}.to_tokens(field_tracks_value);
                quote! {,}.to_tokens(struct_constructor_fields);
            }

            let group_track_ident = format_ident!("group_{}", group_id);
            quote! {let mut #group_track_ident = false;}.to_tokens(group_tracks);
            if let Some(fields) = &group_field_names {
                let fields = fields.join(", ");
                quote! {
                    if !#group_track_ident {
                        return ::std::result::Result::Err(
                            ::syn::Error::new(
                                ::proc_macro2::Span::call_site(),
                                ::std::format!("Exactly one of {} is required.", #fields)
                            )
                        );
                    }
                }
                .to_tokens(non_optional_handling);
            }

            f.contribute_tokens(
                struct_fields_defs,
                field_tracks_type,
                field_tracks_value,
                field_tracker_ident,
                syn::Index::from(*field_index),
                match_branches,
                &group_track_ident,
                &parsed_ident_ident,
                &group_field_names,
                &parse_stream_ident,
                non_optional_handling,
                struct_constructor_fields,
            );

            *field_index += 1;
        }
    }
}

struct Field {
    ident: Ident,
    question: Option<token::Question>,
    colon: token::Colon,
    field_type: Types,
}

impl Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let question = if input.peek(token::Question) {
            Some(input.parse()?)
        } else {
            None
        };
        let colon = input.parse::<token::Colon>()?;
        let field_type = input.parse::<Types>()?;
        Ok(Self {
            ident,
            question,
            colon,
            field_type,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.field_type.to_tokens(tokens);
    }
}

impl Field {
    fn contribute_tokens(
        &self,
        struct_fields_defs: &mut TokenStream,
        field_tracks_type: &mut TokenStream,
        field_tracks_value: &mut TokenStream,
        field_tracker_ident: &Ident,
        field_index: syn::Index,
        match_branches: &mut TokenStream,
        group_track_ident: &Ident,
        parsed_ident_ident: &Ident,
        group_field_names: &Option<Vec<String>>,
        parse_stream_ident: &Ident,
        non_optional_handling: &mut TokenStream,
        struct_constructor_fields: &mut TokenStream,
    ) {
        let ident = &self.ident;
        let ident_string = ident.to_string();
        let typ = self.field_type.to_type_tokens();
        let def = self.field_type.default_tokens();

        quote! {::std::option::Option<#typ>}.to_tokens(field_tracks_type);

        if self.question.is_some() || group_field_names.is_some() {
            quote! {#ident: ::std::option::Option<#typ>}.to_tokens(struct_fields_defs);
            quote! {#ident: #field_tracker_ident.#field_index}.to_tokens(struct_constructor_fields);
        } else {
            quote! {#ident: #typ}.to_tokens(struct_fields_defs);
            quote! {#ident: #field_tracker_ident.#field_index.unwrap()}
                .to_tokens(struct_constructor_fields);
        }

        if let Some(def) = def {
            quote! {::std::option::Option::Some(#def)}.to_tokens(field_tracks_value);
        } else {
            quote! {::std::option::Option::None}.to_tokens(field_tracks_value);
        }

        let err = if let Some(fields) = group_field_names {
            let fields = fields.join(", ");
            quote! {::std::format!("There can only be one of the following fields: {}", #fields)}
        } else {
            quote! {
                if let ::std::option::Option::None = #field_tracker_ident.#field_index {
                    return ::std::result::Result::Err(
                        ::syn::Error::new(
                            ::proc_macro2::Span::call_site(),
                            ::std::format!("{} is a required field.", #ident_string)
                        )
                    );
                }
            }
            .to_tokens(non_optional_handling);
            quote! {::std::format!("Duplicate field: {}", #ident_string)}
        };

        let parse_tokens = self.field_type.to_parse_tokens(parse_stream_ident);

        quote! {
            #ident_string => {
                if #group_track_ident {
                    return ::std::result::Result::Err(
                        ::syn::Error::new_spanned(
                            #parsed_ident_ident,
                            #err
                        )
                    );
                }
                #group_track_ident = true;
                #parse_stream_ident.parse::<::syn::token::Colon>()?;
                #field_tracker_ident.#field_index = Some({#parse_tokens});
            }
        }
        .to_tokens(match_branches);
    }
}

enum Types {
    Str(Ident, Option<(token::Semi, LitStr)>),
    ByteStr(Ident, Option<(token::Semi, LitByteStr)>),
    Struct(Ident),
    Enum(
        token::Enum,
        Path,
        Bracket,
        Vec<(Ident, Option<token::Comma>)>,
        Option<(token::Semi, Ident)>,
    ),
    Map(Ident),
}

impl Parse for Types {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(token::Enum) {
            let tok = input.parse::<token::Enum>()?;
            let enum_path = input.parse::<Path>()?;
            let content;
            let bracket = syn::bracketed!(content in input);

            if content.is_empty() {
                return Err(syn::Error::new_spanned(
                    enum_path,
                    "Enum variants cannot be empty!",
                ));
            }

            let mut options = Vec::new();
            let default = loop {
                let ident = content.parse::<Ident>()?;
                let (comma, has_comma) = if content.peek(token::Comma) {
                    (Some(content.parse::<token::Comma>()?), true)
                } else {
                    (None, false)
                };
                options.push((ident, comma));
                if !has_comma {
                    if content.peek(token::Semi) {
                        let semi = content.parse::<token::Semi>()?;
                        let ident = content.parse::<Ident>()?;
                        if !content.is_empty() {
                            return Err(content.error("Unexpected token!"));
                        }
                        break Some((semi, ident));
                    } else {
                        if !content.is_empty() {
                            return Err(content.error("Unexpected token!"));
                        }
                        break None;
                    }
                }
            };

            Ok(Self::Enum(tok, enum_path, bracket, options, default))
        } else {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "String" => {
                    let default = if input.peek(token::Semi) {
                        let semi = input.parse::<token::Semi>()?;
                        let lit = input.parse::<LitStr>()?;
                        Some((semi, lit))
                    } else {
                        None
                    };
                    Ok(Self::Str(ident, default))
                }
                "ByteString" => {
                    let default = if input.peek(token::Semi) {
                        let semi = input.parse::<token::Semi>()?;
                        let lit = input.parse::<LitByteStr>()?;
                        Some((semi, lit))
                    } else {
                        None
                    };
                    Ok(Self::ByteStr(ident, default))
                }
                "map" => Ok(Self::Map(ident)),
                _ => Ok(Self::Struct(ident)),
            }
        }
    }
}

impl ToTokens for Types {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Str(id, def) => {
                id.to_tokens(tokens);
                if let Some((semi, lit)) = def {
                    semi.to_tokens(tokens);
                    lit.to_tokens(tokens);
                }
            }
            Self::ByteStr(id, def) => {
                id.to_tokens(tokens);
                if let Some((semi, lit)) = def {
                    semi.to_tokens(tokens);
                    lit.to_tokens(tokens);
                }
            }
            Self::Struct(ident) => ident.to_tokens(tokens),
            Self::Enum(id, path, brack, options, def) => {
                id.to_tokens(tokens);
                path.to_tokens(tokens);
                brack.surround(tokens, |tokens| {
                    for (id, comma) in options {
                        id.to_tokens(tokens);
                        if let Some(comma) = comma {
                            comma.to_tokens(tokens);
                        }
                    }
                    if let Some((semi, def)) = def {
                        semi.to_tokens(tokens);
                        def.to_tokens(tokens);
                    }
                })
            }
            Self::Map(ident) => ident.to_tokens(tokens),
        }
    }
}

impl Types {
    fn to_type_tokens(&self) -> TokenStream {
        match self {
            Self::Str(_, _) => quote! {::std::string::String}.to_token_stream(),
            Self::ByteStr(_, _) => quote! {&'static [::std::primitive::u8]}.to_token_stream(),
            Self::Struct(ident) => Path::from(ident.clone()).to_token_stream(),
            Self::Enum(_, path, _, _, _) => path.to_token_stream(),
            Self::Map(_) => {
                quote! {::std::collections::BTreeMap<::std::string::String, ::std::string::String>}
                    .to_token_stream()
            }
        }
    }

    fn default_tokens(&self) -> Option<TokenStream> {
        match self {
            Self::Str(_, d) => d.as_ref().map(|(_, d)| quote! {#d.to_string()}.into_token_stream()),
            Self::ByteStr(_, d) => d.as_ref().map(|(_, d)| quote! {#d}.into_token_stream()),
            Self::Struct(_) => None,
            Self::Enum(_, path, _, _, d) => d
                .as_ref()
                .map(|(_, d)| quote! {#path::#d}.into_token_stream()),
            Self::Map(_) => None,
        }
    }

    fn to_parse_tokens(&self, parse_stream_ident: &Ident) -> TokenStream {
        match &self {
            Self::Str(_, _) => {
                quote! {#parse_stream_ident.parse::<::syn::LitStr>()?.value()}
            }
            Self::ByteStr(_, _) => {
                quote! {#parse_stream_ident.parse::<::syn::LitByteStr>()?.value().leak()}
            }
            Self::Struct(ident) => {
                quote! {#parse_stream_ident.parse::<#ident>()?}
            }
            Self::Enum(_, path, _, options, _) => {
                let mut enum_options_matches = TokenStream::new();
                let mut option_strings = Vec::with_capacity(options.len());
                for (o, _) in options {
                    let ident_string = o.to_string();
                    quote! {#ident_string => #path::#o,}.to_tokens(&mut enum_options_matches);
                    option_strings.push(ident_string);
                }
                let option_strings = option_strings.join(", ");
                quote! {
                    let ident = #parse_stream_ident.parse::<Ident>()?;
                    let ident_string = ident.to_string();
                    match ident_string.as_str() {
                        #enum_options_matches
                        _ => {
                            return ::std::result::Result::Err(
                                ::syn::Error::new_spanned(
                                    ident,
                                    ::std::format!("Must be one of the following: {}", #option_strings)
                                )
                            );
                        }
                    }
                }
            }
            Self::Map(_) => {
                quote! {
                    let mut map = ::std::collections::BTreeMap::<::std::string::String, ::std::string::String>::new();
                    let brace = ::syn::__private::parse_braces(&#parse_stream_ident)?;
                    let content = brace.content;
                    while !content.is_empty() {
                        let key = content.parse::<::syn::Ident>()?;
                        let colon = content.parse::<::syn::token::Colon>()?;
                        let value = content.parse::<::syn::LitStr>()?;
                        map.insert(key.to_string(), value.value());
                        if content.is_empty() {
                            break;
                        }
                        content.parse::<::syn::token::Comma>()?;
                    }
                    map
                }
            }
        }
    }
}
