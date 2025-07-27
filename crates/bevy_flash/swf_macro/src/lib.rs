use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(SwfMaterial)]
pub fn swf_material_derive(input: TokenStream) -> TokenStream {
    // 解析输入的 Rust 代码为语法树
    let ast: DeriveInput = syn::parse(input).unwrap();
    // 提取结构体的名称
    let name = &ast.ident;
    // 判断结构体字段含有transform
    let has_transform_field = if let Data::Struct(data_struct) = ast.data {
        match data_struct.fields {
            Fields::Named(ref fields_named) => fields_named
                .named
                .iter()
                .any(|f| f.ident.as_ref().unwrap() == "transform"),
            _ => false,
        }
    } else {
        false
    };
    if !has_transform_field {
        return quote! {
            compile_error!("derive(SwfMaterial) requires a struct with a `transform` field");
        }
        .into();
    }
    let gen = quote! {
        impl SwfMaterial for #name {
            fn update_swf_material(&mut self, swf_transform: SwfTransform) {
                self.transform = swf_transform
            }
            fn set_blend_key(&mut self,blend_key: BlendMaterialKey) {
                self.blend_key = blend_key
            }
        }
    };
    gen.into()
}
