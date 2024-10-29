use crate::{Number, VMError, Value, ValueRange};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use crate::derive::csv_vec;

impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Number::Int(n) => quote! { Number::Int(#n) },
            Number::Float(n) => quote! { Number::Float(#n) }
        };
        tokens.extend(t)
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            Value::None => {
                quote! {
                    Value::None
                }
            }
            Value::Bool(b) => {
                quote! {
                    Value::Bool(#b)
                }
            }
            Value::Number(n) => {
                quote! {
                    Value::Number(#n)
                }
            }
            Value::String(s) => {
                quote! {
                    Value::String(#s.into())
                }
            }
            Value::List(v) => {
                let values = csv_vec(&v);
                quote! {
                    Value::List(#values)
                }
            }
            Value::Map(map) => {
                let values: Vec<_> = map
                    .into_iter()
                    .map(|(k, v)| {
                        quote! { (#k, #v), }
                    })
                    .collect();
                quote! {
                    Value::Map(IndexMap::from([#(#values)*]))
                }
            }
            Value::Range(r) => {
                quote! {
                    Value::Range(#r)
                }
            }
            Value::Error(e) => {
                quote! {
                    Value::Error(#e)
                }
            }
            Value::Type(r) => {
                quote! {
                    Value::Type(#r)
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
            VMError::EmptyRegister(s) => quote! { VMError::EmptyRegister(#s.into()) },
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
        };
        tokens.extend(t)
    }
}
