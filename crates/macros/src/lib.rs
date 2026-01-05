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

/// Style 派生宏
///
/// 为样式枚举生成以下内容：
/// 1. `{EnumName}Key` - 样式属性键枚举
/// 2. `{EnumName}Update` - 响应式更新枚举（用于 view_id.update_state）
/// 3. `{EnumName}Computed` - 计算后的样式结构体（包含 Option<T> 字段）
/// 4. `{EnumName}StateSelectorExt` - StateSelector 的链式方法 trait
/// 5. `{EnumName}StateSelector` - StateSelector 类型别名
/// 6. `impl {EnumName}` - 添加辅助方法 `update_view`
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

    // 生成类型名称
    let key_enum_name = syn::Ident::new(&format!("{}Key", enum_name), enum_name.span());
    let update_enum_name = syn::Ident::new(&format!("{}Update", enum_name), enum_name.span());
    let computed_struct_name = syn::Ident::new(&format!("{}Computed", enum_name), enum_name.span());
    let trait_name = syn::Ident::new(&format!("{}StateSelectorExt", enum_name), enum_name.span());
    let alias_name = syn::Ident::new(&format!("{}StateSelector", enum_name), enum_name.span());

    // 收集变体
    let mut key_variants = Vec::new();
    let mut update_variants = Vec::new();
    let mut trait_methods = Vec::new();
    let mut impl_methods = Vec::new();

    // Computed 结构体字段
    let mut computed_fields = Vec::new();
    // compute_style 方法的 match 分支
    let mut compute_match_arms = Vec::new();
    // update_view 方法的 match 分支
    let mut update_match_arms = Vec::new();

    for v in variants.iter() {
        // 检查是否有 #[style(skip_attr)]
        let has_skip_attr = v.attrs.iter().any(|a| {
            if a.path().is_ident("style") {
                if let Ok(list) = a.meta.require_list() {
                    return list.tokens.to_string().contains("skip_attr");
                }
            }
            false
        });

        // 检查是否有 #[style(skip_linkfn)]
        let has_skip_linkfn = v.attrs.iter().any(|a| {
            if a.path().is_ident("style") {
                if let Ok(list) = a.meta.require_list() {
                    return list.tokens.to_string().contains("skip_linkfn");
                }
            }
            false
        });

        // skip_attr: 跳过所有生成
        if has_skip_attr {
            continue;
        }

        let variant_ident = &v.ident;
        let snake_name = variant_ident.to_string().to_snake_case();
        let method_name = syn::Ident::new(&snake_name, variant_ident.span());
        let field_name = syn::Ident::new(&snake_name, variant_ident.span());

        // 始终添加 Key 枚举变体
        key_variants.push(quote! { #variant_ident });

        match &v.fields {
            Fields::Unit => {
                // 单元变体暂不处理复杂逻辑
            }
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let ty = &f.unnamed.first().unwrap().ty;

                // Update 枚举
                update_variants.push(quote! { 
                    #variant_ident(flor::view::control_state::ControlState, #ty) 
                });

                // Computed 字段
                computed_fields.push(quote! { pub #field_name: Option<#ty> });

                // compute_style match arm
                // compute_style match arm
                compute_match_arms.push(quote! {
                    #key_enum_name::#variant_ident => {
                        if let #enum_name::#variant_ident(val) = v {
                            computed.#field_name = Some(val.clone());
                        }
                    }
                });

                // update_view match arm
                update_match_arms.push(quote! {
                    #update_enum_name::#variant_ident(state, val) => {
                        style.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(val));
                    }
                });

                if !has_skip_linkfn {
                    trait_methods.push(quote! { fn #method_name(self, value: #ty) -> Self; });
                    impl_methods.push(quote! {
                        fn #method_name(mut self, value: #ty) -> Self {
                            self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(value));
                            self
                        }
                    });
                }
            }
            Fields::Unnamed(f) => {
                let args: Vec<_> = (0..f.unnamed.len()).map(|i| format_ident!("arg{}", i)).collect();
                let args_ty: Vec<_> = f.unnamed.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();

                // Update 枚举
                update_variants.push(quote! { 
                    #variant_ident(flor::view::control_state::ControlState, #(#args_ty),*) 
                });

                // Computed 字段 (Tuple)
                computed_fields.push(quote! { pub #field_name: Option<(#(#args_ty),*)> });

                // compute_style match arm
                compute_match_arms.push(quote! {
                    #key_enum_name::#variant_ident => {
                        if let #enum_name::#variant_ident(#(#args),*) = v {
                            computed.#field_name = Some((#(#args.clone()),*));
                        }
                    }
                });

                // update_view match arm
                update_match_arms.push(quote! {
                    #update_enum_name::#variant_ident(state, #(#args),*) => {
                        style.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
                    }
                });

                if !has_skip_linkfn {
                    trait_methods.push(quote! { fn #method_name(self, #(#trait_args),*) -> Self; });
                    impl_methods.push(quote! {
                        fn #method_name(mut self, #(#trait_args),*) -> Self {
                            self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
                            self
                        }
                    });
                }
            }
            Fields::Named(f) => {
                let args: Vec<_> = f.named.iter().map(|x| x.ident.as_ref().unwrap()).collect();
                let args_ty: Vec<_> = f.named.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();
                let update_fields: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();

                // Update 枚举
                update_variants.push(quote! { 
                    #variant_ident { state: flor::view::control_state::ControlState, #(#update_fields),* } 
                });

                // Computed 字段 (Struct-like tuple? No, Option doesn't support named fields easily inside)
                // Use tuple for computed struct field
                computed_fields.push(quote! { pub #field_name: Option<(#(#args_ty),*)> });

                // compute_style match arm
                compute_match_arms.push(quote! {
                    #key_enum_name::#variant_ident => {
                        if let #enum_name::#variant_ident { #(#args),* } = v {
                            computed.#field_name = Some((#(#args.clone()),*));
                        }
                    }
                });

                // update_view match arm
                update_match_arms.push(quote! {
                    #update_enum_name::#variant_ident { state, #(#args),* } => {
                        style.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident { #(#args),* });
                    }
                });

                if !has_skip_linkfn {
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
    }

    let expanded = quote! {
        // ==========================================
        // Key 枚举
        // ==========================================
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #key_enum_name {
            #(#key_variants),*
        }

        // ==========================================
        // Update 枚举
        // ==========================================
        #[derive(Clone, Debug)]
        pub enum #update_enum_name {
            #(#update_variants),*
        }

        // ==========================================
        // Computed 结构体
        // ==========================================
        #[derive(Clone, Debug, Default)]
        pub struct #computed_struct_name {
            #(#computed_fields),*
        }

        // ==========================================
        // StateSelector 扩展 trait (Chainable + Computed)
        // ==========================================
        pub trait #trait_name: Sized {
            #(#trait_methods)*
            fn compute_style(&self, state: flor::view::control_state::ControlState) -> #computed_struct_name;
        }

        use flor::view::state_selector::StateSelector;
        impl #trait_name for StateSelector<#key_enum_name, #enum_name> {
            #(#impl_methods)*
            
            fn compute_style(&self, state: flor::view::control_state::ControlState) -> #computed_struct_name {
                let mut computed = #computed_struct_name::default();

                // 1. 应用 Normal 状态 (基础样式)
                if let Some(map) = self.styles.get(&flor::view::control_state::ControlState::Normal) {
                    for (k, v) in map {
                        match k {
                            #(#compute_match_arms)*
                            _ => {}
                        }
                    }
                }

                // 2. 如果当前状态不是 Normal，应用当前状态样式 (覆盖)
                if state != flor::view::control_state::ControlState::Normal {
                    if let Some(map) = self.styles.get(&state) {
                         for (k, v) in map {
                            match k {
                                #(#compute_match_arms)*
                                _ => {}
                            }
                        }
                    }
                }
                computed
            }
        }

        // ==========================================
        // Enum 扩展 impl
        // ==========================================
        impl #enum_name {
            pub fn update_view(style: &mut #alias_name, update: #update_enum_name) {
                match update {
                    #(#update_match_arms)*
                }
            }
        }

        // ==========================================
        // 类型别名
        // ==========================================
        pub type #alias_name = StateSelector<#key_enum_name, #enum_name>;
    };

    TokenStream::from(expanded)
}

// =============================================================================
// #[style(ControlName)] 属性宏
// =============================================================================
//
// 用法:
// ```rust
// #[style(Label)]
// pub enum LabelStyle {
//     TextColor(Color),
//     FontSize(f32),
//     FontWeight(FontWeight),
//     ...
// }
// ```
//
// 自动生成:
// 1. 枚举本身 (添加 Clone, Debug derive)
// 2. {EnumName}Key, {EnumName}Update, {EnumName}Computed
// 3. {EnumName}StateSelectorExt trait
// 4. Prop traits (根据类型推断)
// 5. impl ControlName 便捷方法
// =============================================================================

use heck::ToPascalCase;

/// 从类型中提取最后一个路径段的名称
/// 例如: `flor::graphics::base::Color` -> `Color`
///       `f32` -> `f32`
fn get_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            // 获取最后一个路径段
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                quote!(#ty).to_string().replace(" ", "")
            }
        }
        _ => quote!(#ty).to_string().replace(" ", ""),
    }
}

/// 判断是否是 std 预定义类型（flor 已在 prop.rs 中定义）
fn is_std_predefined_type(type_name: &str) -> bool {
    matches!(type_name, 
        // 基本类型
        "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "f32" | "f64" | "char" |
        // 字符串
        "String"
    )
}

/// 判断类型是否是 Copy 类型
fn is_copy_type(type_name: &str) -> bool {
    matches!(type_name, 
        "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "f32" | "f64" | "char"
    )
}

/// 获取类型对应的 Prop trait 路径
/// std 预定义类型返回 flor::view::view_builder::prop::XXXProp
/// 自定义类型返回本地定义的 XXXProp
fn get_prop_trait_path(ty: &syn::Type) -> quote::__private::TokenStream {
    let type_name = get_type_name(ty);

    // 转换为 PascalCase 并添加 Prop 后缀
    let prop_name = format!("{}Prop", type_name.to_pascal_case());

    use syn::spanned::Spanned;
    let prop_ident = syn::Ident::new(&prop_name, ty.span());

    if is_std_predefined_type(&type_name) {
        // std 预定义类型，引用 flor 中的定义
        quote! { flor::view::view_builder::prop::#prop_ident }
    } else {
        // 自定义类型，使用本地定义
        quote! { #prop_ident }
    }
}

/// 获取类型对应的 Prop trait 标识符（仅用于定义时）
fn get_prop_trait_name(ty: &syn::Type) -> syn::Ident {
    use syn::spanned::Spanned;
    let type_name = get_type_name(ty);
    let prop_name = format!("{}Prop", type_name.to_pascal_case());
    syn::Ident::new(&prop_name, ty.span())
}

/// 获取类型对应的 define_prop! 宏调用
/// std 预定义类型返回 None（直接使用 flor 中的定义）
/// 自定义类型返回 Some(TokenStream)
fn get_prop_definition(ty: &syn::Type) -> Option<quote::__private::TokenStream> {
    let type_name = get_type_name(ty);

    // std 预定义类型不需要生成，直接使用 flor 中的定义
    if is_std_predefined_type(&type_name) {
        return None;
    }

    let prop_trait_name = get_prop_trait_name(ty);

    if is_copy_type(&type_name) {
        // Copy 类型
        Some(quote! { flor::define_prop!(copy #prop_trait_name, #ty); })
    } else {
        // 其他类型使用 clone
        Some(quote! { flor::define_prop!(clone #prop_trait_name, #ty); })
    }
}

#[proc_macro_attribute]
pub fn style(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析控件名
    let control_name = parse_macro_input!(attr as syn::Ident);

    // 保留原始 token stream 用于输出
    let original_item: proc_macro2::TokenStream = item.clone().into();

    // 解析枚举定义
    let input = parse_macro_input!(item as DeriveInput);
    let enum_name = &input.ident;

    // 只支持 enum
    let variants = if let Data::Enum(DataEnum { variants, .. }) = &input.data {
        variants
    } else {
        return syn::Error::new_spanned(&input, "style attribute can only be applied to enums")
            .to_compile_error()
            .into();
    };

    // 生成类型名称
    let key_enum_name = format_ident!("{}Key", enum_name);
    let update_enum_name = format_ident!("{}Update", enum_name);
    let computed_struct_name = format_ident!("{}Computed", enum_name);
    let trait_name = format_ident!("{}StateSelectorExt", enum_name);
    let alias_name = format_ident!("{}StateSelector", enum_name);

    // 收集各种生成内容
    let mut key_variants = Vec::new();
    let mut update_variants = Vec::new();
    let mut trait_methods = Vec::new();
    let mut impl_methods = Vec::new();
    let mut computed_fields = Vec::new();
    let mut compute_match_arms = Vec::new();
    let mut update_match_arms = Vec::new();

    // Prop 相关
    let mut prop_definitions = std::collections::HashSet::new();
    let mut prop_defs_tokens = Vec::new();
    let mut control_methods = Vec::new();

    for v in variants.iter() {
        let variant_ident = &v.ident;
        let fields = &v.fields;

        // 检查是否有 #[style(skip)]
        let has_skip = v.attrs.iter().any(|a| {
            if a.path().is_ident("style") {
                if let Ok(list) = a.meta.require_list() {
                    return list.tokens.to_string().contains("skip");
                }
            }
            false
        });

        if has_skip {
            continue;
        }

        let snake_name = variant_ident.to_string().to_snake_case();
        let method_name = syn::Ident::new(&snake_name, variant_ident.span());
        let field_name = syn::Ident::new(&snake_name, variant_ident.span());

        // Key 枚举变体
        key_variants.push(quote! { #variant_ident });

        match fields {
            Fields::Unit => {
                // 单元变体不生成便捷方法
            }
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let ty = &f.unnamed.first().unwrap().ty;

                // Update 枚举
                update_variants.push(quote! { 
                    #variant_ident(flor::view::control_state::ControlState, #ty) 
                });

                // Computed 字段
                computed_fields.push(quote! { pub #field_name: Option<#ty> });

                // compute_style match arm
                compute_match_arms.push(quote! {
                    #key_enum_name::#variant_ident => {
                        if let #enum_name::#variant_ident(val) = v {
                            computed.#field_name = Some(val.clone());
                        }
                    }
                });

                // update_view match arm
                update_match_arms.push(quote! {
                    #update_enum_name::#variant_ident(state, val) => {
                        style.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(val));
                    }
                });

                // StateSelector trait/impl 方法
                trait_methods.push(quote! { fn #method_name(self, value: #ty) -> Self; });
                impl_methods.push(quote! {
                    fn #method_name(mut self, value: #ty) -> Self {
                        self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(value));
                        self
                    }
                });

                // Prop trait 和便捷方法
                let prop_trait = get_prop_trait_path(ty);
                let type_str = quote!(#ty).to_string();
                if !prop_definitions.contains(&type_str) {
                    prop_definitions.insert(type_str);
                    // 只有非 std 类型才需要生成 define_prop!
                    if let Some(def) = get_prop_definition(ty) {
                        prop_defs_tokens.push(def);
                    }
                }

                // 生成控件便捷方法
                control_methods.push(quote! {
                    pub fn #method_name<P: #prop_trait>(self, value: P) -> Self {
                        let view_id = self.view_id;
                        flor::signal::effect::updater_effect::create_updater(
                            move || value.make(),
                            move |v| {
                                view_id.update_state(Box::new(#update_enum_name::#variant_ident(
                                    flor::view::control_state::ControlState::Normal,
                                        v,
                                    )));
                                },
                            );
                            self
                        }
                });
            }
            Fields::Unnamed(f) => {
                // 多参数变体
                let args: Vec<_> = (0..f.unnamed.len()).map(|i| format_ident!("arg{}", i)).collect();
                let args_ty: Vec<_> = f.unnamed.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args.iter().zip(args_ty.iter()).map(|(a, t)| quote! { #a: #t }).collect();

                update_variants.push(quote! { 
                    #variant_ident(flor::view::control_state::ControlState, #(#args_ty),*) 
                });

                computed_fields.push(quote! { pub #field_name: Option<(#(#args_ty),*)> });

                compute_match_arms.push(quote! {
                    #key_enum_name::#variant_ident => {
                        if let #enum_name::#variant_ident(#(#args),*) = v {
                            computed.#field_name = Some((#(#args.clone()),*));
                        }
                    }
                });

                update_match_arms.push(quote! {
                    #update_enum_name::#variant_ident(state, #(#args),*) => {
                        style.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
                    }
                });

                trait_methods.push(quote! { fn #method_name(self, #(#trait_args),*) -> Self; });
                impl_methods.push(quote! {
                    fn #method_name(mut self, #(#trait_args),*) -> Self {
                        self.push(#key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
                        self
                    }
                });
            }
            Fields::Named(_) => {
                // Named 变体暂不生成便捷方法
            }
        }
    }

    let expanded = quote! {
        // ==========================================
        // Prop Traits (自动生成)
        // ==========================================
        #(#prop_defs_tokens)*

        // ==========================================
        // 原始枚举 (保持用户定义不变)
        // ==========================================
        #original_item

        // ==========================================
        // Key 枚举
        // ==========================================
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #key_enum_name {
            #(#key_variants),*
        }

        // ==========================================
        // Update 枚举
        // ==========================================
        #[derive(Debug)]
        pub enum #update_enum_name {
            #(#update_variants),*
        }

        // ==========================================
        // Computed 结构体
        // ==========================================
        #[derive(Clone, Debug, Default)]
        pub struct #computed_struct_name {
            #(#computed_fields),*
        }

        // ==========================================
        // StateSelector 扩展 trait
        // ==========================================
        pub trait #trait_name: Sized {
            #(#trait_methods)*
            fn compute_style(&self, state: flor::view::control_state::ControlState) -> #computed_struct_name;
        }

        use flor::view::state_selector::StateSelector;
        impl #trait_name for StateSelector<#key_enum_name, #enum_name> {
            #(#impl_methods)*
            
            fn compute_style(&self, state: flor::view::control_state::ControlState) -> #computed_struct_name {
                let mut computed = #computed_struct_name::default();

                if let Some(map) = self.styles.get(&flor::view::control_state::ControlState::Normal) {
                    for (k, v) in map {
                        match k {
                            #(#compute_match_arms)*
                            _ => {}
                        }
                    }
                }

                if state != flor::view::control_state::ControlState::Normal {
                    if let Some(map) = self.styles.get(&state) {
                        for (k, v) in map {
                            match k {
                                #(#compute_match_arms)*
                                _ => {}
                            }
                        }
                    }
                }
                computed
            }
        }

        // ==========================================
        // Enum 扩展方法
        // ==========================================
        impl #enum_name {
            pub fn update_view(style: &mut #alias_name, update: #update_enum_name) {
                match update {
                    #(#update_match_arms)*
                }
            }
        }

        // ==========================================
        // 类型别名
        // ==========================================
        pub type #alias_name = flor::view::state_selector::StateSelector<#key_enum_name, #enum_name>;

        // ==========================================
        // 控件便捷方法
        // ==========================================
        impl #control_name {
            #(#control_methods)*
        }
    };

    TokenStream::from(expanded)
}
