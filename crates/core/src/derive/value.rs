use crate::derive::{boxed, csv_vec};
use crate::{Number, ObjectValue, PrimitiveValue, VMError, ValueRange};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for ObjectValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            ObjectValue::Primitive(p) => quote! { ObjectValue::Primitive(#p) },
            ObjectValue::List(v) => {
                let values = csv_vec(v);
                quote! {
                    ObjectValue::List(#values)
                }
            }
            ObjectValue::Tuple(v) => {
                let values = csv_vec(v);
                quote! {
                    ObjectValue::Tuple(#values)
                }
            }
            ObjectValue::Map(map) => {
                let values: Vec<_> = map
                    .into_iter()
                    .map(|(k, v)| {
                        quote! { (#k, #v), }
                    })
                    .collect();
                quote! {
                    ObjectValue::Map(IndexMap::from([#(#values)*]))
                }
            }
            ObjectValue::Object(v) => todo!("Unable to convert {v:?} to tokens"),
            ObjectValue::Enum(i, v, b) => match b {
                None => quote! { ObjectValue::Enum(#i, #v, None) },
                Some(b) => {
                    let b = boxed(b);
                    quote! { ObjectValue::Enum(#i, #v, Some(#b)) }
                },
            }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Number::Int(n) => quote! { Number::Int(#n) },
            Number::Float(n) => quote! { Number::Float(#n) },
        };
        tokens.extend(t)
    }
}

impl ToTokens for PrimitiveValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            PrimitiveValue::None => {
                quote! {
                    PrimitiveValue::None
                }
            }
            PrimitiveValue::Bool(b) => {
                quote! {
                    PrimitiveValue::Bool(#b)
                }
            }
            PrimitiveValue::Number(n) => {
                quote! {
                    PrimitiveValue::Number(#n)
                }
            }
            PrimitiveValue::String(s) => {
                quote! {
                    PrimitiveValue::String(#s.into())
                }
            }
            PrimitiveValue::Range(r) => {
                quote! {
                    PrimitiveValue::Range(#r)
                }
            }
            PrimitiveValue::Error(e) => {
                quote! {
                    PrimitiveValue::Error(#e)
                }
            }
            PrimitiveValue::Type(r) => {
                quote! {
                    PrimitiveValue::Type(#r)
                }
            }
        };
        tokens.extend(t);
    }
}

impl ToTokens for ValueRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let r = match self {
            ValueRange::Int(i) => {
                let s = i.start;
                let e = i.end;
                let i = quote! { #s..#e };
                quote! { ValueRange::Int(#i) }
            }
            ValueRange::Char(c) => {
                let s = c.start;
                let e = c.end;
                let c = quote! { #s..#e };
                quote! { ValueRange::Char(#c) }
            }
        };
        tokens.extend(r)
    }
}

impl ToTokens for VMError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            VMError::RuntimeError(s) => quote! { VMError::RuntimeError(#s.into()) },
            VMError::EmptyStack(s) => quote! { VMError::EmptyRegister(#s.into()) },
            VMError::ConversionError(s) => quote! { VMError::ConversionError(#s.into()) },
            VMError::ScopeDoesNotExist(s) => {
                quote! { VMError::ScopeDoesNotExist(#s.into()) }
            }
            VMError::UnsupportedOperation(s) => {
                quote! { VMError::UnsupportedOperation(#s.into()) }
            }
            VMError::VariableDoesNotExist(s) => {
                quote! { VMError::VariableDoesNotExist(#s.into()) }
            }
            VMError::InvalidModule(s) => quote! { VMError::InvalidModule(#s.into()) },
            VMError::InvalidModuleFunction(s) => {
                quote! { VMError::InvalidModuleFunction(#s.into()) }
            }
            VMError::LifecycleError(s) => quote! { VMError::LifecycleError(#s.into()) },
            VMError::TimeoutError(s) => quote! { VMError::TimeoutError(#s.into()) },
        };
        tokens.extend(t)
    }
}
