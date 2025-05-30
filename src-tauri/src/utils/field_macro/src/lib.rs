use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// 集成所有配置相关功能的宏
/// 1. 提供 load() 方法从全局配置加载字段
/// 2. 实现 Default 以从默认配置获取值
/// 3. 实现 fill_defaults_from 方法用于填充缺失的字段
#[proc_macro_derive(ConfigField)]
pub fn derive_config_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    // 将字段名转换为标识符而不是字符串
    let field_name_ident = format_ident!("{}", name.to_string().to_lowercase());
    
    // 获取结构体的所有字段信息
    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(named_fields) = &data.fields {
            named_fields.named.iter().collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // 生成填充默认值的代码
    let mut fill_default_fields = quote! {};
    
    for field in &fields {
        if let Some(field_name) = &field.ident {
            // 获取字段类型
            let field_type = &field.ty;
            
            let _field_name_str = field_name.to_string();
            let type_str = quote! { #field_type }.to_string();
            
            // 根据字段类型生成不同的填充逻辑
            if type_str.contains("Option") {
                // 对于Option类型字段，使用is_none检查
                fill_default_fields.extend(quote! {
                    if instance.#field_name.is_none() {
                        instance.#field_name = default.#field_name.clone();
                        *was_modified = true;
                    }
                });
            } else if type_str.contains("String") {
                // 对于String类型字段，使用is_empty检查
                fill_default_fields.extend(quote! {
                    if instance.#field_name.is_empty() {
                        instance.#field_name = default.#field_name.clone();
                        *was_modified = true;
                    }
                });
            }
            // 对于布尔类型等其他类型，不需要进行默认值填充检查
        }
    }

    let expanded = quote! {
        impl #name {
            pub fn load() -> Result<#name> {
                crate::utils::config::with_config(|config| {
                    // 使用字段标识符而不是字符串
                    config.#field_name_ident.clone()
                })
            }

            /// 填充缺失的字段，返回是否被修改
            pub fn fill_defaults_from(&mut self, default: &#name, was_modified: &mut bool) {
                let instance = self;
                #fill_default_fields
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