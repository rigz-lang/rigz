// use crate::ast::Scope;

// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionSignature<'vm> {
//     pub arguments: Vec<FunctionArgument<'vm>>,
//     pub return_type: FunctionType,
//     pub self_type: Option<FunctionType>,
//     // todo varargs are only valid for positional arguments
//     pub positional: bool,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionDefinition<'lex> {
//     pub name: &'lex str,
//     pub type_definition: FunctionSignature<'lex>,
//     pub body: Scope<'lex>,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionType {
//     pub rigz_type: RigzType,
//     pub mutable: bool,
// }
//
// impl Into<FunctionType> for RigzType {
//     fn into(self) -> FunctionType {
//         FunctionType::new(self)
//     }
// }
//
// impl FunctionType {
//     pub fn new(rigz_type: RigzType) -> Self {
//         Self {
//             rigz_type,
//             mutable: false,
//         }
//     }
//
//     pub fn mutable(rigz_type: RigzType) -> Self {
//         Self {
//             rigz_type,
//             mutable: true,
//         }
//     }
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionArgument<'vm> {
//     pub name: &'vm str,
//     pub default: Option<Value>,
//     pub function_type: FunctionType,
//     pub var_arg: bool,
// }

// pub mod ast;
pub mod modules;
pub mod prepare;
pub mod runtime;
// pub(crate) mod token;

pub use rigz_ast::*;
pub use runtime::{eval, Runtime, RuntimeError};
