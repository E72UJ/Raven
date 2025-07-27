use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(KeyFrame)]
pub fn key_frame_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = impl_key_frame(&input);

    proc_macro::TokenStream::from(expanded)
}

fn impl_key_frame(ast: &DeriveInput) -> TokenStream {
    // 获取被标准类型的名称
    let name = &ast.ident;

    let gen_code = quote! {
        impl KeyFrame for #name {
            fn time(&self)->f32 {
                self.time
            }
        }
    };

    gen_code
}
