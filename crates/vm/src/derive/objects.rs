use crate::derive::{boxed, csv_vec};
use crate::{CustomType, RigzType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for RigzType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            RigzType::None => quote! { RigzType::None },
            RigzType::Any => quote! { RigzType::Any },
            RigzType::Bool => quote! { RigzType::Bool },
            RigzType::Int => quote! { RigzType::Int },
            RigzType::Float => quote! { RigzType::Float },
            RigzType::Number => quote! { RigzType::Number },
            RigzType::String => quote! { RigzType::String },
            RigzType::Error => quote! { RigzType::Error },
            RigzType::This => quote! { RigzType::This },
            RigzType::Range => quote! { RigzType::Range },
            RigzType::List(t) => {
                let t = boxed(t);
                quote! { RigzType::List(#t) }
            }
            RigzType::Map(k, v) => {
                let k = boxed(k);
                let v = boxed(v);
                quote! { RigzType::Map(#k, #v) }
            }
            RigzType::Type {
                base_type,
                optional,
                can_return_error,
            } => {
                let b = boxed(base_type);
                quote! {
                    RigzType::Type {
                        base_type: #b,
                        optional: #optional,
                        can_return_error: #can_return_error,
                    }
                }
            }
            RigzType::Custom(c) => {
                quote! {
                    RigzType::Custom(#c)
                }
            }
            RigzType::Function(args, ret) => {
                let args = csv_vec(args);
                let ret = boxed(ret);
                quote! {
                    RigzType::Function(#args, #ret)
                }
            }
            RigzType::Union(args) => {
                let args = csv_vec(args);
                quote! {
                    RigzType::Union(#args)
                }
            }
            RigzType::Composite(args) => {
                let args = csv_vec(args);
                quote! {
                    RigzType::Composite(#args)
                }
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for CustomType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CustomType { name, fields } = self;
        let fields: Vec<_> = fields
            .into_iter()
            .map(|(name, ty)| {
                quote! {
                    (#name.into(), #ty),
                }
            })
            .collect();
        tokens.extend(quote! {
            CustomType {
                name: #name.into(),
                fields: vec![#(#fields)*]
            }
        })
    }
}

pub fn rigz_type_to_rust_str(rigz_type: &RigzType) -> Option<String> {
    let type_str = match rigz_type {
        RigzType::None => return None,
        RigzType::Bool => "bool".to_string(),
        RigzType::Int => "i64".to_string(),
        RigzType::Float => "f64".to_string(),
        RigzType::Any => "Value".to_string(),
        RigzType::Type {
            base_type,
            optional,
            can_return_error,
        } => match (base_type.as_ref(), optional, can_return_error) {
            (t, false, false) => return rigz_type_to_rust_str(t),
            (t, true, false) => match rigz_type_to_rust_str(t) {
                None => "Option<()>".to_string(),
                Some(t) => format!("Option<{t}>"),
            },
            (t, false, true) => match rigz_type_to_rust_str(t) {
                None => "Result<(), VMError>".to_string(),
                Some(t) => format!("Result<{t}, VMError>"),
            },
            (t, true, true) => match rigz_type_to_rust_str(t) {
                None => "Result<Option<()>, VMError>".to_string(),
                Some(t) => format!("Result<Option<{t}>, VMError>"),
            },
        },
        RigzType::Custom(_) => "Value".to_string(),
        RigzType::List(v) => {
            let v = rigz_type_to_rust_str(v.as_ref()).expect("None is not valid for list types");
            format!("Vec<{v}>")
        }
        RigzType::Map(k, v) => {
            let k = rigz_type_to_rust_str(k.as_ref()).expect("None is not valid for map key types");
            let v =
                rigz_type_to_rust_str(v.as_ref()).expect("None is not valid for map value types");
            format!("IndexMap<{k}, {v}>")
        }
        t => t.to_string(),
    };
    Some(type_str)
}
