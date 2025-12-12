use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, LitStr};

#[proc_macro]
pub fn color(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let s = lit.value();

    // 尝试在编译阶段解析颜色字符串
    match flor::graphics::base::Color::from_hex_str(&s) {
        Ok(c) => {
            let r = c.r;
            let g = c.g;
            let b = c.b;
            let a = c.a;
            quote! {
                flor::graphics::base::Color::rgba(#r, #g, #b, #a)
            }
            .into()
        }
        Err(e) => {
            let msg = format!("invalid color literal '{}': {}", s, e);
            syn::Error::new(lit.span(), msg).to_compile_error().into()
        }
    }
}
#[proc_macro_derive(Style, attributes(style))]
pub fn style_key_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;

    // 只支持 enum
    let variants = if let Data::Enum(DataEnum { variants, .. }) = input.data {
        variants
    } else {
        panic!("Style can only be derived for enums");
    };

    // 生成 Key 枚举
    let key_enum_name = syn::Ident::new(&format!("{}Key", enum_name), enum_name.span());
    let trait_name = syn::Ident::new(&format!("{}StateSelectorExt", enum_name), enum_name.span());
    let alias_name = syn::Ident::new(&format!("{}StateSelector", enum_name), enum_name.span());

    // 生成 Key 枚举变体
    let key_variants = variants.iter().filter_map(|v| {
        if v.attrs.iter().any(|a| a.path().is_ident("skip_attr")) {
            None
        } else {
            Some(&v.ident)
        }
    });

    // trait 方法签名 & impl 方法体
    let mut trait_methods = Vec::new();
    let mut impl_methods = Vec::new();

    for v in variants.iter() {
        if v.attrs.iter().any(|a| a.path().is_ident("skip_attr") || a.path().is_ident("skip_linkfn")) {
            continue;
        }

        let variant_ident = &v.ident;
        let method_name = syn::Ident::new(&variant_ident.to_string().to_snake_case(), variant_ident.span());

        match &v.fields {
            Fields::Unit => {
                // 单元不生成
            }
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let ty = &f.unnamed.first().unwrap().ty;
                trait_methods.push(quote! { fn #method_name(self, value: #ty) -> Self; });
                impl_methods.push(quote! {
                    fn #method_name(mut self, value: #ty) -> Self {
                        self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(value));
                        self
                    }
                });
            }
            Fields::Unnamed(f) => {
                let args: Vec<_> = (0..f.unnamed.len()).map(|i| format_ident!("arg{}", i)).collect();
                let args_ty: Vec<_> = f.unnamed.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();
                impl_methods.push(quote! {
                    fn #method_name(mut self, #(#trait_args),*) -> Self {
                        self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
                        self
                    }
                });
                trait_methods.push(quote! { fn #method_name(self, #(#trait_args),*) -> Self; });
            }
            Fields::Named(f) => {
                let args: Vec<_> = f.named.iter().map(|x| x.ident.as_ref().unwrap()).collect();
                let args_ty: Vec<_> = f.named.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();
                trait_methods.push(quote! { fn #method_name(self, #(#trait_args),*) -> Self; });
                impl_methods.push(quote! {
                    fn #method_name(mut self, #(#trait_args),*) -> Self {
                        self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident { #(#args),* });
                        self
                    }
                });
            }
        }
    }

    let expanded = quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #key_enum_name {
            #(#key_variants),*
        }

        pub trait #trait_name: Sized {
            #(#trait_methods)*
        }

        impl #trait_name for StateSelector<#key_enum_name, #enum_name> {
            #(#impl_methods)*
        }

        use flor::view::style::style_selector::StateSelector;
        pub type #alias_name = StateSelector<#key_enum_name, #enum_name>;
    };

    TokenStream::from(expanded)
}
