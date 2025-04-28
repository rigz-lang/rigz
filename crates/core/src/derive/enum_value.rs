use proc_macro2::TokenStream;
use quote::ToTokens;
use crate::EnumDeclaration;

impl ToTokens for EnumDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        todo!()
    }
}