use proc_macro2::*;

use crate::{BinaryOperation, UnaryOperation};
use quote::*;

impl ToTokens for UnaryOperation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            UnaryOperation::Neg => quote! { UnaryOperation::Neg },
            UnaryOperation::Not => quote! { UnaryOperation::Not },
            UnaryOperation::Reverse => quote! { UnaryOperation::Reverse },
            UnaryOperation::Print => quote! { UnaryOperation::Print },
            UnaryOperation::EPrint => quote! { UnaryOperation::EPrint },
            UnaryOperation::PrintLn => quote! { UnaryOperation::PrintLn },
            UnaryOperation::EPrintLn => quote! { UnaryOperation::EPrintLn },
        };
        tokens.extend(t);
    }
}

impl ToTokens for BinaryOperation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            BinaryOperation::Add => quote! { BinaryOperation::Add },
            BinaryOperation::Sub => quote! { BinaryOperation::Sub },
            BinaryOperation::Mul => quote! { BinaryOperation::Mul },
            BinaryOperation::Div => quote! { BinaryOperation::Div },
            BinaryOperation::Rem => quote! { BinaryOperation::Rem },
            BinaryOperation::Shr => quote! { BinaryOperation::Shr },
            BinaryOperation::Shl => quote! { BinaryOperation::Shl },
            BinaryOperation::BitOr => quote! { BinaryOperation::BitOr },
            BinaryOperation::BitAnd => quote! { BinaryOperation::BitAnd },
            BinaryOperation::BitXor => quote! { BinaryOperation::BitXor },
            BinaryOperation::Or => quote! { BinaryOperation::Or },
            BinaryOperation::And => quote! { BinaryOperation::And },
            BinaryOperation::Xor => quote! { BinaryOperation::Xor },
            BinaryOperation::Eq => quote! { BinaryOperation::Eq },
            BinaryOperation::Neq => quote! { BinaryOperation::Neq },
            BinaryOperation::Gte => quote! { BinaryOperation::Gte },
            BinaryOperation::Gt => quote! { BinaryOperation::Gt },
            BinaryOperation::Lt => quote! { BinaryOperation::Lt },
            BinaryOperation::Lte => quote! { BinaryOperation::Lte },
            BinaryOperation::Elvis => quote! { BinaryOperation::Elvis },
        };
        tokens.extend(t);
    }
}
