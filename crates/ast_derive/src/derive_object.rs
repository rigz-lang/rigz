use crate::{convert_response, convert_type_for_arg, rigz_type_to_return_type, setup_call_args};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use rigz_ast::{FunctionDeclaration, ObjectDefinition, Parser, ParserOptions};
use rigz_core::derive::Tokens;
use rigz_core::RigzType;
use syn::parse::{Parse, ParseStream};
use syn::{token, ItemStruct, LitStr, Token, Type, Visibility};

enum ObjectArg {
    Ident(Ident),
    Struct(ItemStruct),
}

pub(crate) struct DeriveObject {
    parent: LitStr,
    definition: ObjectArg,
    literal: LitStr,
    display: bool,
}

impl Parse for DeriveObject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent = input.parse()?;
        input.parse::<Token![,]>()?;
        let definition = if input.peek(token::Struct) {
            ObjectArg::Struct(input.parse()?)
        } else {
            ObjectArg::Ident(input.parse()?)
        };
        input.parse::<Token![,]>()?;
        Ok(DeriveObject {
            parent,
            definition,
            literal: input.parse()?,
            display: true,
        })
    }
}

impl ToTokens for DeriveObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.full_definition())
    }
}

fn type_to_rigz_type(rust_type: &Type) -> RigzType {
    match rust_type {
        Type::Path(t) => {
            if t.qself.is_none() {
                if let Some(last) = t.path.segments.last() {
                    match last.ident.to_string().as_str() {
                        "i8" | "i16" | "i32" | "i64" => return RigzType::Int,
                        "f32" | "f64" => return RigzType::Float,
                        "bool" => return RigzType::Bool,
                        "String" => return RigzType::String,
                        _ => {}
                    }
                }
            }
        }
        t => {}
    }
    RigzType::Any
}

impl DeriveObject {
    fn full_definition(&self) -> Tokens {
        let parent = &self.parent;
        let (id, base, lit) = match &self.definition {
            ObjectArg::Ident(i) => (i, quote! {}, self.literal.value()),
            ObjectArg::Struct(s) => {
                let id = &s.ident;
                let pub_fields = s
                    .fields
                    .iter()
                    .filter(|f| matches!(f.vis, Visibility::Public(_)))
                    .map(|f| {
                        let name = f
                            .ident
                            .as_ref()
                            .expect("Anonymous Fields are not allowed in generated structs")
                            .to_string();
                        let rt = type_to_rigz_type(&f.ty);
                        (name, rt)
                    });
                let fields = pub_fields
                    .clone()
                    .map(|(name, rt)| quote! { (#name.to_string(), #rt) });
                let type_info = quote! {
                    impl rigz_core::WithTypeInfo for #id {
                        fn rigz_type(&self) -> rigz_core::RigzType {
                            rigz_core::RigzType::Custom(rigz_core::CustomType {
                                name: Self::name().to_string(),
                                fields: vec![#(#fields, )*],
                            })
                        }
                    }
                };

                let mut lit = self.literal.value();
                let name = id.to_string();
                let pub_fields = pub_fields
                    .map(|(name, rt)| {
                        if matches!(rt, RigzType::Any) {
                            format!("attr {name}")
                        } else {
                            format!("attr {name}, {}", rt)
                        }
                    })
                    .fold(String::new(), |mut res, next| {
                        res.push_str(next.as_str());
                        res.push('\n');
                        res
                    });
                lit = lit.replace(
                    format!("object {}", &name).as_str(),
                    format!("object {}::{}\n{}", parent.value(), &name, pub_fields).as_str(),
                );

                (
                    id,
                    quote! {
                        #[derive(derivative::Derivative, serde::Serialize, serde::Deserialize)]
                        #[derive(Clone)]
                        #[derivative(Debug, Default, Hash, PartialOrd, PartialEq)]
                        pub #s

                        #type_info
                    },
                    lit,
                )
            }
        };

        let base = if self.display {
            quote! {
                #base

                impl std::fmt::Display for #id {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{self:?}")
                    }
                }
            }
        } else {
            base
        };

        let name = id.to_string();
        let mut obj_def = Parser::prepare(lit.as_str(), ParserOptions::default());
        let obj_def = obj_def
            .parse_object_definition()
            .expect("failed to parse object definition");
        let definition = quote! {
            impl rigz_core::Definition for #id {
                fn name() -> &'static str {
                    concat!(#parent, "::", #name)
                }

                fn trait_definition() -> &'static str {
                    #lit
                }
            }

            impl rigz_ast::ParsedObject for #id {
                fn object_definition() -> ObjectDefinition
                where
                    Self: Sized,
                {
                    #obj_def
                }
            }
        };

        let impl_object = impl_object(id, &obj_def);

        quote! {
            #base

            #definition

            #impl_object
        }
    }
}

fn impl_object(name: &Ident, object_definition: &ObjectDefinition) -> Tokens {
    let CustomTrait {
        ext,
        mutf,
        statf,
        trait_def,
    } = custom_trait(name, object_definition);

    quote! {
        #[typetag::serde]
        impl rigz_core::Object for #name {
            #ext

            #mutf

            #statf
        }

        #trait_def
    }
}

struct CustomTrait {
    ext: Option<Tokens>,
    mutf: Option<Tokens>,
    statf: Option<Tokens>,
    trait_def: Tokens,
}

fn custom_trait(name: &Ident, object_definition: &ObjectDefinition) -> CustomTrait {
    let funcs: Vec<_> = object_definition
        .functions
        .iter()
        .filter_map(|f| match f {
            FunctionDeclaration::Declaration {
                name,
                type_definition,
            } => Some((name, type_definition)),
            FunctionDeclaration::Definition(_) => None,
        })
        .collect();

    let mut stat_funcs = Vec::new();
    let mut mut_funcs = Vec::new();
    let mut ext_funcs = Vec::new();

    let trait_methods = funcs
        .iter()
        .map(|(name, sig)| {
            let (mut args, fn_name) = match &sig.self_type {
                Some(s) => {
                    let mut_str = if s.mutable { "mut_" } else { "" };
                    let (base, n) = if let RigzType::This = s.rigz_type {
                        (
                            quote! { self },
                            Ident::new(format!("{mut_str}{name}").as_str(), Span::call_site()),
                        )
                    } else {
                        panic!(
                            "Non Self extensions are not supported for Objects yet {:?}",
                            s.rigz_type
                        );
                        let arg = match rigz_type_to_return_type(&s.rigz_type) {
                            None => quote! { ObjectValue },
                            Some(t) => quote! { #t },
                        };
                        (
                            quote! { value: #arg },
                            Ident::new(
                                format!("{mut_str}{}", s.rigz_type.to_string().to_lowercase())
                                    .as_str(),
                                Span::call_site(),
                            ),
                        )
                    };
                    let base = if s.mutable {
                        quote! { &mut #base }
                    } else {
                        quote! { &#base }
                    };
                    (vec![base], n)
                }
                None => {
                    let fn_name = Ident::new(format!("static_{name}").as_str(), Span::call_site());
                    (vec![], fn_name)
                }
            };

            let mut var_arg = false;
            args.extend(sig.arguments.iter().map(|arg| {
                let ident = Ident::new(arg.name.as_str(), Span::call_site());
                let rt = match rigz_type_to_return_type(&arg.function_type.rigz_type) {
                    None => quote! { ObjectValue },
                    Some(t) => quote! { #t },
                };
                var_arg |= arg.var_arg;
                if var_arg {
                    quote! { #ident: Vec<#rt> }
                } else {
                    quote! { #ident: #rt }
                }
            }));

            let (call_args, setup_args, var_args) = setup_call_args(sig);
            let base_args = match var_args {
                None => quote! { let [#(#call_args)*] = args.take()?; },
                Some(s) => {
                    let (base, var) = call_args.split_at(s);
                    let (_, var_args) = sig.arguments.split_at(s);
                    let mut call_var = quote! {};
                    for v in var_args {
                        if v.function_type.rigz_type == RigzType::default() {
                            continue;
                        }
                        let name = Ident::new(v.name.as_str(), Span::call_site());
                        if let Some((v, _)) = convert_type_for_arg(
                            quote! { n },
                            &v.function_type.rigz_type,
                            v.function_type.mutable,
                        ) {
                            call_var = quote! {
                                #call_var
                                let #name = #name.into_iter().map(|n| #v).collect();
                            };
                        }
                    }
                    quote! {
                        let ([#(#base)*], [#(#var)*]) = args.var_args()?;
                        #call_var
                    }
                }
            };

            match &sig.self_type {
                None => {
                    let method_call =
                        convert_response(quote! { Self::#fn_name(#(#call_args)*) }, sig);
                    stat_funcs.push(quote! {
                        #name => {
                            #base_args
                            #(#setup_args)*
                            #method_call
                        }
                    });
                }
                Some(ft) if ft.mutable => {
                    let method_call =
                        convert_response(quote! { self.#fn_name(#(#call_args)*) }, sig);
                    mut_funcs.push(quote! {
                        #name => {
                            #base_args
                            #(#setup_args)*
                            #method_call
                        }
                    });
                }
                Some(_) => {
                    let method_call =
                        convert_response(quote! { self.#fn_name(#(#call_args)*) }, sig);
                    ext_funcs.push(quote! {
                        #name => {
                            #base_args
                            #(#setup_args)*
                            #method_call
                        }
                    });
                }
            }

            let ret = match rigz_type_to_return_type(&sig.return_type.rigz_type) {
                None => None,
                Some(s) => Some(quote! { -> #s }),
            };
            if sig.self_type.is_none() {
                quote! {
                    fn #fn_name(#(#args, )*) #ret where Self: Sized;
                }
            } else {
                quote! {
                    fn #fn_name(#(#args, )*) #ret;
                }
            }
        })
        .collect::<Vec<_>>();

    let ext = if ext_funcs.is_empty() {
        None
    } else {
        Some(quote! {
            fn call_extension(&self, function: String, args: RigzArgs) -> Result<ObjectValue, VMError> {
                match function.as_str() {
                    #(#ext_funcs)*
                    _ => {
                        Err(VMError::UnsupportedOperation(format!(
                            "{self:?} does not implement `call_extension` - {function}"
                        )))
                    }
                }
            }
        })
    };

    let mutf = if mut_funcs.is_empty() {
        None
    } else {
        Some(quote! {
            fn call_mutable_extension(
                &mut self,
                function: String,
                args: RigzArgs,
            ) -> Result<Option<ObjectValue>, VMError>
            {
                match function.as_str() {
                    #(#mut_funcs)*
                    _ => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "{self:?} does not implement `call_mutable_extension` - {function}"
                        )))
                    }
                }
                Ok(None)
            }
        })
    };

    let statf = if stat_funcs.is_empty() {
        None
    } else {
        Some(quote! {
            fn call(function: String, args: RigzArgs) -> Result<ObjectValue, VMError> where Self: Sized {
                match function.as_str() {
                    #(#stat_funcs)*
                    _ => {
                        Err(VMError::UnsupportedOperation(format!(
                            "{} does not implement `call` - {function}", Self::name()
                        )))
                    }
                }
            }
        })
    };

    let trait_name = Ident::new(format!("{}Object", name).as_str(), Span::call_site());
    let trait_def = quote! {
        trait #trait_name {
            #(#trait_methods)*
        }
    };

    CustomTrait {
        ext,
        mutf,
        statf,
        trait_def,
    }
}
