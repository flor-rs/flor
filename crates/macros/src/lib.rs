mod shared;

use crate::shared::flor_crate;
use flor_base::types::Color;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, LitStr};

#[proc_macro]
pub fn color(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let s = lit.value();

    // 返回 syn:Ident
    let flor_crate = flor_crate();

    // 尝试在编译阶段解析颜色字符串
    match Color::from_hex_str(&s) {
        Ok(c) => {
            let r = c.r;
            let g = c.g;
            let b = c.b;
            let a = c.a;
            quote! {
                #flor_crate::types::Color::rgba(#r, #g, #b, #a)
            }
            .into()
        }
        Err(e) => {
            let msg = format!("invalid color literal '{}': {}", s, e);
            syn::Error::new(lit.span(), msg).to_compile_error().into()
        }
    }
}

/// Resolver 派生宏
///
/// 为样式枚举生成以下内容：
/// 1. `{EnumName}Key` - 样式属性键枚举
/// 2. `{EnumName}Update` - 响应式更新枚举（可选，`update_view = false` 跳过）
/// 3. `{EnumName}Computed` - 计算后的样式结构体（可选，`computed = false` 跳过）
/// 4. `{EnumName}ResolverExt` - Resolver 的链式方法 trait
/// 5. `{EnumName}Resolver` - Resolver 类型别名（需指定 `data = Type`）
/// 6. `impl {EnumName}` - 添加辅助方法 `update_view`（可选）
///
/// ## 基础用法
///
/// ```rust
/// #[derive(Clone, Debug, Resolver)]
/// pub enum LabelStyle {
///     TextColor(Color),
///     FontSize(f32),
/// }
/// ```
///
/// ## 完整配置示例（如 Layout）
///
/// ```rust
/// #[derive(Clone, Debug, Resolver)]
/// #[resolver(update_view = false, computed = false, computed_fn = false, data = taffy::Style)]
/// pub enum Layout {
///     Display(Display),
///     Size(Size<Dimension>),
/// }
/// ```
///
/// ## 参数说明
///
/// - `update_view = false` - 不生成 Update 枚举和 update_view 方法
/// - `computed = false` - 不生成 Computed 结构体
/// - `computed_fn = false` - 不生成 computed_xxx 独立函数
/// - `data = Type` - 指定 Resolver 的 D 泛型类型，生成类型别名
/// - `control = ControlName` - 显式指定控件名（用于 StyleBuilder）
/// - `builder = false` - 跳过 StyleBuilder 生成
/// - `default = false` - 不生成 Resolver 的 Default 实现
#[proc_macro_derive(Resolver, attributes(resolver))]
pub fn resolver_derive(input: TokenStream) -> TokenStream {
    generate_resolver_impl(input)
}

/// 核心代码生成逻辑
fn generate_resolver_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident.clone();
    let enum_name_str = enum_name.to_string();

    let flor_crate = flor_crate();

    // 解析枚举级别的 #[resolver(...)] 属性
    let mut generate_update_view = true; // 默认生成 update_view
    let mut generate_computed = true; // 默认生成 computed 结构体
    let mut generate_computed_fn = true; // 默认生成 computed_xxx 函数
    let mut generate_default = true; // 默认生成 Default 实现
    let mut data_type: Option<syn::Type> = None; // data = Type
    let mut control_type: Option<syn::Type> = None; // control = ControlName
    let mut field_name_attr: Option<syn::Ident> = None; // field = field_name

    for attr in input.attrs.iter() {
        if !attr.path().is_ident("resolver") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("update_view") {
                // #[resolver(update_view = false)]
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitBool = meta.input.parse()?;
                generate_update_view = value.value;
            } else if meta.path.is_ident("computed") {
                // #[resolver(computed = false)]
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitBool = meta.input.parse()?;
                generate_computed = value.value;
            } else if meta.path.is_ident("computed_fn") {
                // #[resolver(computed_fn = false)]
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitBool = meta.input.parse()?;
                generate_computed_fn = value.value;
            } else if meta.path.is_ident("data") {
                // #[resolver(data = taffy::Style)]
                let _: syn::Token![=] = meta.input.parse()?;
                let ty: syn::Type = meta.input.parse()?;
                data_type = Some(ty);
            } else if meta.path.is_ident("default") {
                // #[resolver(default = false)]
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitBool = meta.input.parse()?;
                generate_default = value.value;
            } else if meta.path.is_ident("control") {
                // #[resolver(control = TextInput)]
                let _: syn::Token![=] = meta.input.parse()?;
                let ty: syn::Type = meta.input.parse()?;
                control_type = Some(ty);
            } else if meta.path.is_ident("field") {
                // #[resolver(field = style)]
                let _: syn::Token![=] = meta.input.parse()?;
                let ident: syn::Ident = meta.input.parse()?;
                field_name_attr = Some(ident);
            }
            Ok(())
        });
    }

    // Validate: control and field must both be present or both be absent
    match (&control_type, &field_name_attr) {
        (Some(_), None) => {
            panic!("Resolver macro error: 'control' attribute is specified but 'field' is missing. \
                    Both 'control' and 'field' must be specified together to generate StyleBuilder implementation. \
                    Example: #[resolver(control = TextInput, field = style)]");
        }
        (None, Some(_)) => {
            panic!("Resolver macro error: 'field' attribute is specified but 'control' is missing. \
                    Both 'control' and 'field' must be specified together to generate StyleBuilder implementation. \
                    Example: #[resolver(control = TextInput, field = style)]");
        }
        _ => {}
    }

    // 只支持 enum
    let variants = if let Data::Enum(DataEnum { variants, .. }) = input.data {
        variants
    } else {
        panic!("Resolver can only be derived for enums");
    };

    // 生成类型名称
    let key_enum_name = syn::Ident::new(&format!("{}Key", enum_name), enum_name.span());
    let update_enum_name = syn::Ident::new(&format!("{}Update", enum_name), enum_name.span());
    let computed_struct_name = syn::Ident::new(&format!("{}Computed", enum_name), enum_name.span());
    let trait_name = syn::Ident::new(&format!("{}ResolverExt", enum_name), enum_name.span());
    let alias_name = syn::Ident::new(&format!("{}Resolver", enum_name), enum_name.span());

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
        // 检查是否有 #[resolver(skip_attr)]
        let has_skip_attr = v.attrs.iter().any(|a| {
            if a.path().is_ident("resolver") {
                if let Ok(list) = a.meta.require_list() {
                    return list.tokens.to_string().contains("skip_attr");
                }
            }
            false
        });

        // 检查是否有 #[resolver(skip_linkfn)]
        let has_skip_linkfn = v.attrs.iter().any(|a| {
            if a.path().is_ident("resolver") {
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
                    #variant_ident(#flor_crate::view::control_state::ControlState, #ty)
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
                        resolver.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(val));
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
                let args: Vec<_> = (0..f.unnamed.len())
                    .map(|i| format_ident!("arg{}", i))
                    .collect();
                let args_ty: Vec<_> = f.unnamed.iter().map(|x| &x.ty).collect();
                let trait_args: Vec<_> = args
                    .iter()
                    .zip(args_ty.iter())
                    .map(|(a, t)| quote! { #a: #t })
                    .collect();

                // Update 枚举
                update_variants.push(quote! {
                    #variant_ident(#flor_crate::view::control_state::ControlState, #(#args_ty),*)
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
                        resolver.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident(#(#args),*));
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
                let trait_args: Vec<_> = args
                    .iter()
                    .zip(args_ty.iter())
                    .map(|(a, t)| quote! { #a: #t })
                    .collect();
                let update_fields: Vec<_> = args
                    .iter()
                    .zip(args_ty.iter())
                    .map(|(a, t)| quote! { #a: #t })
                    .collect();

                // Update 枚举
                update_variants.push(quote! {
                    #variant_ident { state: #flor_crate::view::control_state::ControlState, #(#update_fields),* }
                });

                // Computed 字段
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
                        resolver.update(state, #key_enum_name::#variant_ident, #enum_name::#variant_ident { #(#args),* });
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

    // Update 枚举（条件生成）
    let update_enum_code = if generate_update_view {
        quote! {
            // ==========================================
            // Update 枚举
            // ==========================================
            #[derive(Clone, Debug)]
            pub enum #update_enum_name {
                #(#update_variants),*
            }
        }
    } else {
        quote! {}
    };

    // update_view 方法（条件生成）
    let update_view_impl = if generate_update_view {
        quote! {
            // ==========================================
            // Enum 扩展 impl
            // ==========================================
            impl #enum_name {
                pub fn update_view<D, F>(
                    resolver: &mut #flor_crate::view::resolver::Resolver<#key_enum_name, #enum_name, D, F>,
                    update: #update_enum_name
                )
                where
                    D: Clone,
                    F: for<'a> Fn(
                        &#flor_crate::view::resolver::UnitResolver,
                        #flor_crate::view::control_state::ControlState,
                        &#flor_crate::rustc_hash::FxHashMap<#flor_crate::view::control_state::ControlState, #flor_crate::rustc_hash::FxHashMap<#key_enum_name, #enum_name>>
                    ) -> D,
                {
                    match update {
                        #(#update_match_arms)*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    // 生成 computed_xxx 独立函数名（snake_case）
    let computed_fn_name = syn::Ident::new(
        &format!("computed_{}", enum_name_str.to_snake_case()),
        enum_name.span(),
    );

    // Computed 结构体（条件生成）
    let computed_struct_code = if generate_computed {
        quote! {
            // ==========================================
            // Computed 结构体
            // ==========================================
            #[derive(Clone, Debug, Default)]
            pub struct #computed_struct_name {
                #(#computed_fields),*
            }
        }
    } else {
        quote! {}
    };

    // computed_xxx 独立函数（条件生成，带优化的 clone 逻辑）
    // 需要 computed 结构体存在才能生成
    let computed_fn_code = if generate_computed && generate_computed_fn {
        quote! {
            // ==========================================
            // computed_xxx 独立函数
            // ==========================================
            pub fn #computed_fn_name(
                _unit_resolver: &#flor_crate::view::resolver::UnitResolver,
                state: #flor_crate::view::control_state::ControlState,
                state_variants: &#flor_crate::rustc_hash::FxHashMap<#flor_crate::view::control_state::ControlState, #flor_crate::rustc_hash::FxHashMap<#key_enum_name, #enum_name>>,
            ) -> #computed_struct_name {
                let mut computed = #computed_struct_name::default();

                // 预先获取 Specific 状态的 Map 引用，用于优化 clone
                let specific_variants = if state == #flor_crate::view::control_state::ControlState::Normal {
                    None
                } else {
                    state_variants.get(&state)
                };

                // 1. 应用 Normal 状态 (基础样式)
                if let Some(normal_map) = state_variants.get(&#flor_crate::view::control_state::ControlState::Normal) {
                    for (k, v) in normal_map.iter() {
                        // 优化：如果 Specific 层有相同的 key，跳过 Normal 层的 clone
                        if specific_variants.map_or(false, |s| s.contains_key(k)) {
                            continue;
                        }
                        match k {
                            #(#compute_match_arms)*
                            _ => {}
                        }
                    }
                }

                // 2. 应用 Specific (Current State) 层
                if let Some(map) = specific_variants {
                    for (k, v) in map.iter() {
                        match k {
                            #(#compute_match_arms)*
                            _ => {}
                        }
                    }
                }
                computed
            }
        }
    } else {
        quote! {}
    };

    // 类型别名（始终生成）
    // D 类型：如果指定了 data，使用指定的类型；否则使用生成的 Computed 结构体
    // F 使用默认泛型参数（函数指针类型），也支持传入闭包
    let type_alias_code = if let Some(ref data_ty) = data_type {
        quote! {
            // ==========================================
            // Resolver 类型别名
            // ==========================================
            pub type #alias_name<F = fn(
                &#flor_crate::view::resolver::UnitResolver,
                #flor_crate::view::control_state::ControlState,
                &#flor_crate::rustc_hash::FxHashMap<#flor_crate::view::control_state::ControlState, #flor_crate::rustc_hash::FxHashMap<#key_enum_name, #enum_name>>
            ) -> #data_ty> = #flor_crate::view::resolver::Resolver<
                #key_enum_name,
                #enum_name,
                #data_ty,
                F
            >;
        }
    } else if generate_computed {
        // 默认使用生成的 Computed 结构体作为 D
        quote! {
            // ==========================================
            // Resolver 类型别名
            // ==========================================
            pub type #alias_name<F = fn(
                &#flor_crate::view::resolver::UnitResolver,
                #flor_crate::view::control_state::ControlState,
                &#flor_crate::rustc_hash::FxHashMap<#flor_crate::view::control_state::ControlState, #flor_crate::rustc_hash::FxHashMap<#key_enum_name, #enum_name>>
            ) -> #computed_struct_name> = #flor_crate::view::resolver::Resolver<
                #key_enum_name,
                #enum_name,
                #computed_struct_name,
                F
            >;
        }
    } else {
        // computed = false 且未指定 data，不生成类型别名
        quote! {}
    };

    // StyleBuilder implementation (when both control and field are specified)
    let style_builder_code = if let (Some(control_ty), Some(field_ident)) =
        (&control_type, &field_name_attr)
    {
        quote! {
            // ==========================================
            // StyleBuilder implementation for control
            // ==========================================
            impl #flor_crate::view::view_builder::style_builder::StyleBuilder<#alias_name> for #control_ty {
                fn style(mut self, style_fn: impl Fn(#alias_name) -> #alias_name) -> Self {
                    self.#field_ident = style_fn(self.#field_ident.clone().normal());
                    self
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        // ==========================================
        // Key 枚举
        // ==========================================
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #key_enum_name {
            #(#key_variants),*
        }

        #update_enum_code

        #computed_struct_code

        #computed_fn_code

        // ==========================================
        // Resolver 扩展 trait (Chainable)
        // ==========================================
        pub trait #trait_name: Sized {
            #(#trait_methods)*
        }

        impl<D, F> #trait_name for #flor_crate::view::resolver::Resolver<#key_enum_name, #enum_name, D, F>
        where
            D: Clone,
            F: for<'a> Fn(
                &#flor_crate::view::resolver::UnitResolver,
                #flor_crate::view::control_state::ControlState,
                &#flor_crate::rustc_hash::FxHashMap<#flor_crate::view::control_state::ControlState, #flor_crate::rustc_hash::FxHashMap<#key_enum_name, #enum_name>>
            ) -> D,
        {
            #(#impl_methods)*
        }

        #update_view_impl

        #type_alias_code

        #style_builder_code
    };

    TokenStream::from(expanded)
}
