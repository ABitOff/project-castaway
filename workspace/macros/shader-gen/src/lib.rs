extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result,
};

parseable::parseable! {
    BinaryInfo {
        entry_point: String,
        #[exclusive_group] {
            file: String,
            binary: ByteString,
        },
    }

    SourceInfo {
        #[exclusive_group] {
            file: String,
            code: String,
        },
        entry_point: String; "main",
        shader_stage: enum ::shaderc::ShaderKind[Vertex, Fragment, Compute, Geometry, TessControl, TessEvaluation,
            SpirvAssembly, RayGeneration, AnyHit, ClosestHit, Miss, Intersection, Callable, Task, Mesh;
            InferFromSource],
        language: enum ::shaderc::SourceLanguage[GLSL, HLSL; GLSL],
        spirv_version: enum ::shaderc::SpirvVersion[V1_0, V1_1, V1_2, V1_3, V1_4, V1_5, V1_6; V1_6],
        optimization: enum ::shaderc::OptimizationLevel[Zero, Size, Performance; Performance],
        macros?: map,
    }

    Config {
        #[exclusive_group] {
            source: SourceInfo,
            binary: BinaryInfo,
        },
    }
}

struct ShaderStructInfo {
    struct_name: Ident,
    config: Config,
}

impl Parse for ShaderStructInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_name = input.parse::<Ident>()?;
        let config = input.parse::<Config>()?;
        Ok(ShaderStructInfo {
            struct_name,
            config,
        })
    }
}

#[proc_macro]
pub fn include_shader(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(shader)
        .map_err(syn::Error::into_compile_error)
        .map_or_else(Into::into, Into::into)
}

fn shader(args: ShaderStructInfo) -> Result<TokenStream> {
    let ident = args.struct_name;
    args.config;
    Ok(quote! {struct #ident {}}.into())
}
