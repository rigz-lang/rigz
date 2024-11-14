use quote::{quote, ToTokens};

pub type Tokens = proc_macro2::TokenStream;

mod lifecycle;
mod objects;
mod operations;
mod value;

pub use objects::rigz_type_to_rust_str;

pub fn csv_vec<T: ToTokens>(values: &Vec<T>) -> Tokens {
    let values: Vec<_> = values.into_iter().map(|v| quote! { #v, }).collect();
    quote! { vec![#(#values)*] }
}

pub fn csv_tuple_vec<A: ToTokens, T: ToTokens>(values: &Vec<(A, T)>) -> Tokens {
    let values: Vec<_> = values
        .into_iter()
        .map(|(a, v)| quote! { (#a, #v), })
        .collect();
    quote! { vec![#(#values)*] }
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
