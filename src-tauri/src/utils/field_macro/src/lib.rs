use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};



#[proc_macro_derive(ConfigField)]
pub fn derive_config_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    // 将字段名转换为标识符而不是字符串
    let field_name_ident = format_ident!("{}", name.to_string().to_lowercase());
    
    let expanded = quote! {
        impl #name {
            pub fn load() -> Result<#name> {
                crate::utils::config::with_config(|config| {
                    // 使用字段标识符而不是字符串
                    config.#field_name_ident.clone()
                })
            }
        }
        impl Default for #name {
            fn default() -> Self {
                crate::utils::config::DEFAULT_VALUES.#field_name_ident.clone()
            }
        }
    };
    TokenStream::from(expanded)
}