use quote::{quote, ToTokens};

pub type Tokens = proc_macro2::TokenStream;

mod instructions;
mod lifecycle;
mod objects;
mod value;

pub use objects::rigz_type_to_rust_str;

pub fn csv_vec<T: ToTokens>(values: &Vec<T>) -> Tokens {
    quote! { vec![#(#values, )*] }
}

pub fn option<T: ToTokens>(value: &Option<T>) -> Tokens {
    match value {
        None => quote! { None },
        Some(s) => {
            quote! { Some(#s) }
        }
    }
}

pub fn boxed<T: ToTokens>(value: &Box<T>) -> Tokens {
    quote! { Box::new(#value) }
}
