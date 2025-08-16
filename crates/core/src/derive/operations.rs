use proc_macro2::*;

use crate::{BinaryAssignOperation, BinaryOperation, UnaryOperation};
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

impl ToTokens for BinaryAssignOperation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            BinaryAssignOperation::Add => quote! { BinaryAssignOperation::Add },
            BinaryAssignOperation::Sub => quote! { BinaryAssignOperation::Sub },
            BinaryAssignOperation::Mul => quote! { BinaryAssignOperation::Mul },
            BinaryAssignOperation::Div => quote! { BinaryAssignOperation::Div },
            BinaryAssignOperation::Rem => quote! { BinaryAssignOperation::Rem },
            BinaryAssignOperation::Shr => quote! { BinaryAssignOperation::Shr },
            BinaryAssignOperation::Shl => quote! { BinaryAssignOperation::Shl },
            BinaryAssignOperation::BitOr => quote! { BinaryAssignOperation::BitOr },
            BinaryAssignOperation::BitAnd => quote! { BinaryAssignOperation::BitAnd },
            BinaryAssignOperation::BitXor => quote! { BinaryAssignOperation::BitXor },
            BinaryAssignOperation::Or => quote! { BinaryAssignOperation::Or },
            BinaryAssignOperation::And => quote! { BinaryAssignOperation::And },
            BinaryAssignOperation::Xor => quote! { BinaryAssignOperation::Xor }
        };
        tokens.extend(t);
    }
}
