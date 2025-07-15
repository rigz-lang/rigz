use crate::derive::{csv_tuple_vec, csv_vec};
use crate::EnumDeclaration;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for EnumDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumDeclaration { name, variants } = self;
        let vars = csv_tuple_vec(variants);
        tokens.extend(quote! {
            EnumDeclaration {
                name: #name.to_string(),
                variants: #vars,
            }
        })
    }
}
